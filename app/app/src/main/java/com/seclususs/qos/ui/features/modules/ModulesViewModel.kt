package com.seclususs.qos.ui.features.modules

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.model.SystemStatus
import com.seclususs.qos.domain.repository.ConfigRepository
import com.seclususs.qos.domain.repository.DaemonRepository
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ModulesViewModel @Inject constructor(
    private val configRepository: ConfigRepository,
    private val daemonRepository: DaemonRepository,
    private val appPreferencesRepository: PreferencesRepository
) : ViewModel() {
    private val _state = MutableStateFlow(ModulesState())
    val state: StateFlow<ModulesState> = _state.asStateFlow()

    init {
        _state.collectPolling(
            scope = viewModelScope, intervalMs = 1500L
        ) {
            if (_state.value.processingModules.isEmpty()) {
                refreshInternal()
            }
        }
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
                    val success = configRepository.updateConfig(newConfig)
                    if (success) appPreferencesRepository.setNeedsReboot(true)
                    _state.update {
                        it.copy(
                            config = if (success) newConfig else it.config,
                            processingModules = it.processingModules - event.type
                        )
                    }
                }
            }

            is ModulesEvent.ToggleModuleExpansion -> _state.update {
                it.copy(expandedModule = if (it.expandedModule == event.type) null else event.type)
            }

            is ModulesEvent.RefreshStatus -> viewModelScope.launch { refreshInternal() }
        }
    }

    private suspend fun refreshInternal() {
        coroutineScope {
            val daemonExistsDeferred = async { daemonRepository.checkDaemonExists() }
            val configExistsDeferred = async { configRepository.checkConfigExists() }
            if (!daemonExistsDeferred.await()) {
                _state.update { it.copy(systemStatus = SystemStatus.DAEMON_MISSING) }
                return@coroutineScope
            }
            if (!configExistsDeferred.await()) {
                _state.update { it.copy(systemStatus = SystemStatus.CONFIG_MISSING) }
                return@coroutineScope
            }
            val config = configRepository.getConfig()
            _state.update {
                it.copy(systemStatus = SystemStatus.OK, config = config)
            }
        }
    }
}