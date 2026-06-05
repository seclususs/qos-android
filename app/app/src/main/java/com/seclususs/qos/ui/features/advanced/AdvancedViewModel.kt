package com.seclususs.qos.ui.features.advanced

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.model.SystemStatus
import com.seclususs.qos.domain.model.applyCpuLimits
import com.seclususs.qos.domain.model.applyStorageLimits
import com.seclususs.qos.domain.model.clearCpuLimits
import com.seclususs.qos.domain.model.clearStorageLimits
import com.seclususs.qos.domain.model.decodeToMap
import com.seclususs.qos.domain.model.encodeToString
import com.seclususs.qos.domain.model.updateCpuLimits
import com.seclususs.qos.domain.model.updateStorageLimits
import com.seclususs.qos.domain.repository.ConfigRepository
import com.seclususs.qos.domain.repository.DaemonRepository
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AdvancedViewModel @Inject constructor(
    private val configRepository: ConfigRepository,
    private val daemonRepository: DaemonRepository,
    private val appPreferencesRepository: PreferencesRepository
) : ViewModel() {
    private val _state = MutableStateFlow(AdvancedState())
    val state: StateFlow<AdvancedState> = _state.asStateFlow()

    init {
        appPreferencesRepository.cachedCpuLimitsFlow.onEach { encoded ->
            _state.update { it.copy(cachedCpuValues = encoded.decodeToMap()) }
        }.launchIn(viewModelScope)
        appPreferencesRepository.cachedStorageLimitsFlow.onEach { encoded ->
            _state.update { it.copy(cachedStorageValues = encoded.decodeToMap()) }
        }.launchIn(viewModelScope)
        appPreferencesRepository.advancedCpuEnabledFlow.onEach { enabled ->
            _state.update { it.copy(cpuLimitsEnabled = enabled) }
        }.launchIn(viewModelScope)
        appPreferencesRepository.advancedStorageEnabledFlow.onEach { enabled ->
            _state.update { it.copy(storageLimitsEnabled = enabled) }
        }.launchIn(viewModelScope)
        _state.collectPolling(scope = viewModelScope, intervalMs = 1500L) {
            if (!_state.value.isProcessingCpuToggle && !_state.value.isProcessingStorageToggle) {
                refreshInternal()
            }
        }
    }

    fun onEvent(event: AdvancedEvent) {
        when (event) {
            is AdvancedEvent.ToggleCpu -> handleToggleCpu(event.enabled)
            is AdvancedEvent.ToggleStorage -> handleToggleStorage(event.enabled)
            is AdvancedEvent.ShowSheet -> _state.update { it.copy(activeSheet = event.type) }
            is AdvancedEvent.SaveAndHideSheet -> {
                if (event.cpuValues != null) applyCpuConfig(event.cpuValues)
                else if (event.storageValues != null) applyStorageConfig(event.storageValues)
                _state.update { it.copy(activeSheet = null) }
            }

            is AdvancedEvent.RefreshStatus -> viewModelScope.launch { refreshInternal() }
        }
    }

    private fun handleToggleCpu(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingCpuToggle = true) }
            appPreferencesRepository.setAdvancedCpuEnabled(enabled)
            val currentConfig = configRepository.getConfig()
            val newConfig = if (enabled) {
                currentConfig.applyCpuLimits(_state.value.cachedCpuValues)
            } else {
                currentConfig.clearCpuLimits()
            }
            val success = configRepository.updateConfig(newConfig)
            if (success) appPreferencesRepository.setNeedsReboot(true)
            _state.update {
                it.copy(
                    isProcessingCpuToggle = false, config = if (success) newConfig else it.config
                )
            }
        }
    }

    private fun handleToggleStorage(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingStorageToggle = true) }
            appPreferencesRepository.setAdvancedStorageEnabled(enabled)
            val currentConfig = configRepository.getConfig()
            val newConfig = if (enabled) {
                currentConfig.applyStorageLimits(_state.value.cachedStorageValues)
            } else {
                currentConfig.clearStorageLimits()
            }
            val success = configRepository.updateConfig(newConfig)
            if (success) appPreferencesRepository.setNeedsReboot(true)
            _state.update {
                it.copy(
                    isProcessingStorageToggle = false,
                    config = if (success) newConfig else it.config
                )
            }
        }
    }

    private fun applyCpuConfig(cpuValues: Map<String, String>) {
        viewModelScope.launch {
            appPreferencesRepository.setCachedCpuLimits(cpuValues.encodeToString())
            if (_state.value.cpuLimitsEnabled) {
                val currentConfig = configRepository.getConfig()
                val newConfig = currentConfig.updateCpuLimits(cpuValues)
                val success = configRepository.updateConfig(newConfig)
                if (success) appPreferencesRepository.setNeedsReboot(true)
                _state.update { it.copy(config = if (success) newConfig else it.config) }
            }
        }
    }

    private fun applyStorageConfig(storageValues: Map<String, String>) {
        viewModelScope.launch {
            appPreferencesRepository.setCachedStorageLimits(storageValues.encodeToString())
            if (_state.value.storageLimitsEnabled) {
                val currentConfig = configRepository.getConfig()
                val newConfig = currentConfig.updateStorageLimits(storageValues)
                val success = configRepository.updateConfig(newConfig)
                if (success) appPreferencesRepository.setNeedsReboot(true)
                _state.update { it.copy(config = if (success) newConfig else it.config) }
            }
        }
    }

    private suspend fun refreshInternal() = coroutineScope {
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