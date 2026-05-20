package com.seclususs.qos.ui.features.modules

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.R
import com.seclususs.qos.domain.usecase.CheckConfigUseCase
import com.seclususs.qos.domain.usecase.DaemonStatusUseCase
import com.seclususs.qos.domain.usecase.GetConfigUseCase
import com.seclususs.qos.domain.usecase.ToggleDaemonUseCase
import com.seclususs.qos.domain.usecase.UpdateConfigUseCase
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
    private val getConfigUseCase: GetConfigUseCase,
    private val updateConfigUseCase: UpdateConfigUseCase,
    private val toggleDaemonUseCase: ToggleDaemonUseCase,
    private val daemonStatusUseCase: DaemonStatusUseCase,
    private val checkConfigExistsUseCase: CheckConfigUseCase
) : ViewModel() {

    private val _state = MutableStateFlow(ModulesState())
    val state: StateFlow<ModulesState> = _state.asStateFlow()
    private var pollingJob: Job? = null

    init {
        viewModelScope.launch {
            _state.subscriptionCount.collect { count ->
                if (count > 0) {
                    startPolling()
                } else {
                    stopPolling()
                }
            }
        }
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
                    _state.update {
                        it.copy(processingModules = it.processingModules + event.type)
                    }

                    delay(400)

                    val currentConfig = _state.value.config
                    val newConfig = when (event.type) {
                        ModuleType.BLOCKER -> currentConfig.copy(blockerEnabled = event.enabled)
                        ModuleType.CLEANER -> currentConfig.copy(cleanerEnabled = event.enabled)
                        ModuleType.CPU -> currentConfig.copy(cpuEnabled = event.enabled)
                        ModuleType.STORAGE -> currentConfig.copy(storageEnabled = event.enabled)
                        ModuleType.TWEAKS -> currentConfig.copy(tweaksEnabled = event.enabled)
                    }

                    val success = updateConfigUseCase(newConfig)

                    if (success) {
                        toggleDaemonUseCase.restart()

                        _state.update {
                            it.copy(
                                config = newConfig,
                                snackbarMessageResId = R.string.module_update_success,
                                snackbarIsError = false,
                                snackbarVisible = true
                            )
                        }
                    } else {
                        _state.update {
                            it.copy(
                                snackbarMessageResId = R.string.module_update_error,
                                snackbarIsError = true,
                                snackbarVisible = true
                            )
                        }
                    }

                    _state.update {
                        it.copy(processingModules = it.processingModules - event.type)
                    }

                    delay(3000)
                    _state.update { it.copy(snackbarVisible = false) }
                }
            }

            is ModulesEvent.ShowModuleDetails -> {
                _state.update { it.copy(selectedModuleForDetails = event.type) }
            }

            is ModulesEvent.DismissModuleDetails -> {
                _state.update { it.copy(selectedModuleForDetails = null) }
            }

            is ModulesEvent.RefreshStatus -> {
                viewModelScope.launch { refreshInternal() }
            }

            is ModulesEvent.DismissSnackbar -> {
                _state.update { it.copy(snackbarVisible = false) }
            }
        }
    }

    private suspend fun refreshInternal() {
        val daemonExists = daemonStatusUseCase.checkDaemonExists()
        if (!daemonExists) {
            _state.update { it.copy(isDaemonMissing = true, isConfigMissing = false) }
            return
        }

        val configExists = checkConfigExistsUseCase()
        if (!configExists) {
            _state.update { it.copy(isDaemonMissing = false, isConfigMissing = true) }
            return
        }

        val config = getConfigUseCase()
        _state.update {
            it.copy(isDaemonMissing = false, isConfigMissing = false, config = config)
        }
    }
}