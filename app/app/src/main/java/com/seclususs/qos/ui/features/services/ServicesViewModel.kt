package com.seclususs.qos.ui.features.services

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.domain.usecase.DaemonStatusUseCase
import com.seclususs.qos.domain.usecase.ToggleDaemonUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ServicesViewModel @Inject constructor(
    private val toggleDaemonUseCase: ToggleDaemonUseCase,
    private val daemonStatusUseCase: DaemonStatusUseCase
) : ViewModel() {

    private val _state = MutableStateFlow(ServicesState())
    val state: StateFlow<ServicesState> = _state.asStateFlow()

    private var isTransitioning = false

    init {
        viewModelScope.launch {
            refreshInternal()
            while (true) {
                delay(1000)
                if (!isTransitioning) {
                    refreshInternal()
                }
            }
        }
    }

    fun onEvent(event: ServicesEvent) {
        when (event) {
            is ServicesEvent.OnStartClicked -> {
                if (isTransitioning) return
                viewModelScope.launch {
                    isTransitioning = true
                    _state.update { it.copy(status = DaemonStatus.STARTING) }
                    toggleDaemonUseCase.start()
                    delay(1200)
                    refreshInternal()
                    isTransitioning = false
                }
            }

            is ServicesEvent.OnStopClicked -> {
                if (isTransitioning) return
                viewModelScope.launch {
                    isTransitioning = true
                    _state.update { it.copy(status = DaemonStatus.STOPPING) }
                    toggleDaemonUseCase.stop()
                    delay(1200)
                    refreshInternal()
                    isTransitioning = false
                }
            }

            is ServicesEvent.OnRestartClicked -> {
                if (isTransitioning) return
                viewModelScope.launch {
                    isTransitioning = true
                    _state.update { it.copy(status = DaemonStatus.RESTARTING) }
                    toggleDaemonUseCase.restart()
                    delay(1500)
                    refreshInternal()
                    isTransitioning = false
                }
            }

            is ServicesEvent.RefreshMetrics -> {
                if (!isTransitioning) {
                    viewModelScope.launch {
                        refreshInternal()
                    }
                }
            }
        }
    }

    private suspend fun refreshInternal() {
        val exists = daemonStatusUseCase.checkDaemonExists()
        if (!exists) {
            _state.update {
                it.copy(
                    status = DaemonStatus.MISSING,
                    pid = "-",
                    cpuUsage = "0%",
                    ramUsage = "0 MB",
                    uptime = "00:00:00",
                    cpuProgress = 0f,
                    ramProgress = 0f
                )
            }
            return
        }

        val isRunning = daemonStatusUseCase.isRunning()
        val pid = daemonStatusUseCase.getPid()
        val metrics = daemonStatusUseCase.getMetrics(pid)

        val cpuRaw = metrics.cpuUsage.replace("%", "").trim().toFloatOrNull() ?: 0f
        val cpuProg = (cpuRaw / 100f).coerceIn(0f, 1f)

        val ramPercentRegex = Regex("\\((.*?)%\\)")
        val ramPercentMatch = ramPercentRegex.find(metrics.ramUsage)

        val ramProg = if (ramPercentMatch != null) {
            val percentValue = ramPercentMatch.groupValues[1].toFloatOrNull() ?: 0f
            (percentValue / 100f).coerceIn(0f, 1f)
        } else {
            0f
        }

        val displayRam = metrics.ramUsage.split("(")[0].trim()

        val finalStatus = if (isRunning) DaemonStatus.ACTIVE else DaemonStatus.INACTIVE

        _state.update {
            it.copy(
                status = finalStatus,
                pid = pid,
                cpuUsage = metrics.cpuUsage,
                ramUsage = displayRam,
                uptime = metrics.uptime,
                cpuProgress = cpuProg,
                ramProgress = ramProg
            )
        }
    }
}