package com.seclususs.qos.ui.features.services

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.usecase.DaemonUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ServicesViewModel @Inject constructor(
    private val daemonUseCase: DaemonUseCase
) : ViewModel() {

    private val _state = MutableStateFlow(ServicesState())
    val state: StateFlow<ServicesState> = _state.asStateFlow()

    private var isTransitioning = false
    private var pollingJob: Job? = null

    init {
        _state.collectPolling(
            scope = viewModelScope,
            onStart = { startPolling() },
            onStop = { stopPolling() })
    }

    private fun startPolling() {
        if (pollingJob?.isActive == true) return
        pollingJob = viewModelScope.launch {
            refreshInternal()
            while (true) {
                delay(1000)
                if (!isTransitioning) {
                    refreshInternal()
                }
            }
        }
    }

    private fun stopPolling() {
        pollingJob?.cancel()
        pollingJob = null
    }

    fun onEvent(event: ServicesEvent) {
        when (event) {
            is ServicesEvent.OnStartClicked -> handleToggle(DaemonStatus.STARTING) { daemonUseCase.start() }
            is ServicesEvent.OnStopClicked -> handleToggle(DaemonStatus.STOPPING) { daemonUseCase.stop() }
            is ServicesEvent.OnRestartClicked -> handleToggle(DaemonStatus.RESTARTING) { daemonUseCase.restart() }
            is ServicesEvent.RefreshMetrics -> if (!isTransitioning) viewModelScope.launch { refreshInternal() }
        }
    }

    private fun handleToggle(targetStatus: DaemonStatus, action: suspend () -> Unit) {
        if (isTransitioning) return
        viewModelScope.launch {
            isTransitioning = true
            _state.update { it.copy(status = targetStatus) }
            action()
            delay(1200)
            refreshInternal()
            isTransitioning = false
        }
    }

    private suspend fun refreshInternal() {
        val exists = daemonUseCase.checkExists()
        if (!exists) {
            resetStateTo(DaemonStatus.MISSING)
            return
        }

        val pid = daemonUseCase.getPid()
        if (pid == "-") {
            resetStateTo(DaemonStatus.INACTIVE)
            return
        }

        val metrics = daemonUseCase.getMetrics(pid)
        val cpuRaw = metrics.cpuUsage.removeSuffix("%").trim().toFloatOrNull() ?: 0f
        val cpuProg = (cpuRaw / 100f).coerceIn(0f, 1f)
        val percentStr = metrics.ramUsage.substringAfter("(", "").substringBefore("%", "").trim()
        val ramProg = (percentStr.toFloatOrNull() ?: 0f) / 100f
        val displayRam = metrics.ramUsage.substringBefore("(").trim()

        _state.update {
            it.copy(
                status = DaemonStatus.ACTIVE,
                pid = pid,
                cpuUsage = metrics.cpuUsage,
                ramUsage = displayRam,
                uptime = metrics.uptime,
                cpuProgress = cpuProg,
                ramProgress = ramProg
            )
        }
    }

    private fun resetStateTo(status: DaemonStatus) {
        _state.update { ServicesState(status = status) }
    }
}