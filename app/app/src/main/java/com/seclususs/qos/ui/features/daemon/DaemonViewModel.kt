package com.seclususs.qos.ui.features.daemon

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.repository.DaemonRepository
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject
import kotlin.time.Duration.Companion.milliseconds

@HiltViewModel
class DaemonViewModel @Inject constructor(
    private val daemonRepository: DaemonRepository,
    private val appPreferencesRepository: PreferencesRepository
) : ViewModel() {

    private val _state = MutableStateFlow(DaemonState())
    val state: StateFlow<DaemonState> = _state.asStateFlow()
    private var isTransitioning = false

    init {
        appPreferencesRepository.needsRebootFlow.onEach { needs ->
            _state.update { it.copy(needsReboot = needs) }
        }.launchIn(viewModelScope)
        _state.collectPolling(
            scope = viewModelScope, intervalMs = 800L
        ) {
            if (!isTransitioning) refreshInternal()
        }
    }

    fun onEvent(event: DaemonEvent) {
        when (event) {
            is DaemonEvent.OnStopClicked -> handleStopDaemon()
            is DaemonEvent.OnRebootClicked -> {
                viewModelScope.launch {
                    appPreferencesRepository.setNeedsReboot(false)
                    daemonRepository.rebootDevice()
                }
            }

            is DaemonEvent.RefreshInfo -> {
                if (!isTransitioning) viewModelScope.launch { refreshInternal() }
            }
        }
    }

    private fun handleStopDaemon() {
        if (isTransitioning) return
        viewModelScope.launch {
            isTransitioning = true
            _state.update { it.copy(status = DaemonStatus.STOPPING) }
            delay(3000.milliseconds)
            daemonRepository.stopDaemon()
            refreshInternal()
            isTransitioning = false
        }
    }

    private suspend fun refreshInternal() {
        val exists = daemonRepository.checkDaemonExists()
        if (!exists) {
            resetStateTo(DaemonStatus.MISSING)
            return
        }
        val pid = daemonRepository.getDaemonPid()?.takeIf { it != "-" } ?: run {
            resetStateTo(DaemonStatus.INACTIVE)
            return
        }
        val info = daemonRepository.getDaemonInfo(pid)
        _state.update {
            it.copy(
                status = DaemonStatus.ACTIVE, pid = info.pid, uptime = info.uptime
            )
        }
    }

    private fun resetStateTo(status: DaemonStatus) {
        _state.update {
            it.copy(
                status = status, pid = "-", uptime = "-"
            )
        }
    }
}