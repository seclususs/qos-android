package com.seclususs.qos.ui.features.modules

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.usecase.ConfigUseCase
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
class ModulesViewModel @Inject constructor(
    private val configUseCase: ConfigUseCase, private val daemonUseCase: DaemonUseCase
) : ViewModel() {

    private val _state = MutableStateFlow(ModulesState())
    val state: StateFlow<ModulesState> = _state.asStateFlow()
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
                delay(1500)
                if (_state.value.processingModules.isEmpty()) {
                    refreshInternal()
                }
            }
        }
    }

    private fun stopPolling() {
        pollingJob?.cancel()
        pollingJob = null
    }

    fun onEvent(event: ModulesEvent) {
        when (event) {
            is ModulesEvent.ToggleModule -> {
                viewModelScope.launch {
                    _state.update { it.copy(processingModules = it.processingModules + event.type) }

                    val currentConfig = _state.value.config
                    val newConfig = when (event.type) {
                        ModuleType.BLOCKER -> currentConfig.copy(blockerEnabled = event.enabled)
                        ModuleType.CLEANER -> currentConfig.copy(cleanerEnabled = event.enabled)
                        ModuleType.CPU -> currentConfig.copy(cpuEnabled = event.enabled)
                        ModuleType.STORAGE -> currentConfig.copy(storageEnabled = event.enabled)
                        ModuleType.TWEAKS -> currentConfig.copy(tweaksEnabled = event.enabled)
                    }

                    val success = configUseCase.update(newConfig)
                    if (success) {
                        daemonUseCase.restart()
                        _state.update { it.copy(config = newConfig) }
                    }
                    _state.update { it.copy(processingModules = it.processingModules - event.type) }
                }
            }

            is ModulesEvent.ShowModuleDetails -> _state.update { it.copy(selectedModuleForDetails = event.type) }
            is ModulesEvent.DismissModuleDetails -> _state.update { it.copy(selectedModuleForDetails = null) }
            is ModulesEvent.RefreshStatus -> viewModelScope.launch { refreshInternal() }
        }
    }

    private suspend fun refreshInternal() {
        val daemonExists = daemonUseCase.checkExists()
        if (!daemonExists) {
            _state.update { it.copy(isDaemonMissing = true, isConfigMissing = false) }
            return
        }
        val configExists = configUseCase.checkExists()
        if (!configExists) {
            _state.update { it.copy(isDaemonMissing = false, isConfigMissing = true) }
            return
        }
        val config = configUseCase.get()
        _state.update {
            it.copy(isDaemonMissing = false, isConfigMissing = false, config = config)
        }
    }
}