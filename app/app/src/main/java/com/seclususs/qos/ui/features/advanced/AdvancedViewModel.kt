package com.seclususs.qos.ui.features.advanced

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.core.utils.collectPolling
import com.seclususs.qos.domain.model.ConfigKeys
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.repository.ConfigRepository
import com.seclususs.qos.domain.repository.DaemonRepository
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

private fun Map<String, String>.getLong(key: String, default: Long? = null): Long? =
    this[key]?.toLongOrNull() ?: default

private fun Map<String, String>.getUpdatedLong(key: String, current: Long?): Long? =
    if (this.containsKey(key)) this[key]?.toLongOrNull() else current

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
            _state.update {
                it.copy(cachedCpuValues = encoded.decodeToMap())
            }
        }.launchIn(viewModelScope)
        appPreferencesRepository.cachedStorageLimitsFlow.onEach { encoded ->
            _state.update {
                it.copy(cachedStorageValues = encoded.decodeToMap())
            }
        }.launchIn(viewModelScope)
        _state.collectPolling(
            scope = viewModelScope, intervalMs = 1500L
        ) {
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
                if (event.cpuValues != null) {
                    applyCpuConfig(event.cpuValues)
                } else if (event.storageValues != null) {
                    applyStorageConfig(event.storageValues)
                }
                _state.update { it.copy(activeSheet = null) }
            }

            is AdvancedEvent.RefreshStatus -> viewModelScope.launch { refreshInternal() }
        }
    }

    private fun handleToggleCpu(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingCpuToggle = true) }
            val currentConfig = configRepository.getConfig()
            val newConfig: QosConfig

            if (enabled) {
                val cached = _state.value.cachedCpuValues
                val default = QosConfig()
                newConfig = currentConfig.copy(
                    minLatencyNs = cached.getLong(ConfigKeys.CPU_LATENCY_MIN, default.minLatencyNs),
                    maxLatencyNs = cached.getLong(ConfigKeys.CPU_LATENCY_MAX, default.maxLatencyNs),
                    minGranularityNs = cached.getLong(
                        ConfigKeys.CPU_GRANULARITY_MIN, default.minGranularityNs
                    ),
                    maxGranularityNs = cached.getLong(
                        ConfigKeys.CPU_GRANULARITY_MAX, default.maxGranularityNs
                    ),
                    minWakeupNs = cached.getLong(ConfigKeys.CPU_WAKEUP_MIN, default.minWakeupNs),
                    maxWakeupNs = cached.getLong(ConfigKeys.CPU_WAKEUP_MAX, default.maxWakeupNs),
                    minMigrationCost = cached.getLong(
                        ConfigKeys.CPU_MIGRATION_COST_MIN, default.minMigrationCost
                    ),
                    maxMigrationCost = cached.getLong(
                        ConfigKeys.CPU_MIGRATION_COST_MAX, default.maxMigrationCost
                    ),
                    minWaltInitPct = cached.getLong(
                        ConfigKeys.CPU_WALT_INIT_MIN, default.minWaltInitPct
                    ),
                    maxWaltInitPct = cached.getLong(
                        ConfigKeys.CPU_WALT_INIT_MAX, default.maxWaltInitPct
                    ),
                    minUclampMin = cached.getLong(
                        ConfigKeys.CPU_UCLAMP_MIN_MIN, default.minUclampMin
                    ),
                    maxUclampMin = cached.getLong(
                        ConfigKeys.CPU_UCLAMP_MIN_MAX, default.maxUclampMin
                    )
                )
            } else {
                appPreferencesRepository.setCachedCpuLimits(
                    currentConfig.toCpuMap().encodeToString()
                )
                newConfig = currentConfig.copy(
                    minLatencyNs = null,
                    maxLatencyNs = null,
                    minGranularityNs = null,
                    maxGranularityNs = null,
                    minWakeupNs = null,
                    maxWakeupNs = null,
                    minMigrationCost = null,
                    maxMigrationCost = null,
                    minWaltInitPct = null,
                    maxWaltInitPct = null,
                    minUclampMin = null,
                    maxUclampMin = null
                )
            }

            val success = configRepository.updateConfig(newConfig)
            if (success) {
                appPreferencesRepository.setNeedsReboot(true)
                _state.update {
                    it.copy(
                        config = newConfig, cpuLimitsEnabled = enabled
                    )
                }
            }
            _state.update { it.copy(isProcessingCpuToggle = false) }
        }
    }

    private fun handleToggleStorage(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingStorageToggle = true) }
            val currentConfig = configRepository.getConfig()
            val newConfig: QosConfig

            if (enabled) {
                val cached = _state.value.cachedStorageValues
                val default = QosConfig()
                newConfig = currentConfig.copy(
                    minReadAhead = cached.getLong(
                        ConfigKeys.STORAGE_READ_AHEAD_MIN, default.minReadAhead
                    ), maxReadAhead = cached.getLong(
                        ConfigKeys.STORAGE_READ_AHEAD_MAX, default.maxReadAhead
                    ), minNrRequests = cached.getLong(
                        ConfigKeys.STORAGE_NR_REQUESTS_MIN, default.minNrRequests
                    ), maxNrRequests = cached.getLong(
                        ConfigKeys.STORAGE_NR_REQUESTS_MAX, default.maxNrRequests
                    )
                )
            } else {
                appPreferencesRepository.setCachedStorageLimits(
                    currentConfig.toStorageMap().encodeToString()
                )
                newConfig = currentConfig.copy(
                    minReadAhead = null,
                    maxReadAhead = null,
                    minNrRequests = null,
                    maxNrRequests = null
                )
            }

            val success = configRepository.updateConfig(newConfig)
            if (success) {
                appPreferencesRepository.setNeedsReboot(true)
                _state.update {
                    it.copy(
                        config = newConfig, storageLimitsEnabled = enabled
                    )
                }
            }
            _state.update { it.copy(isProcessingStorageToggle = false) }
        }
    }

    private fun applyCpuConfig(cpuValues: Map<String, String>) {
        if (cpuValues.isEmpty()) return
        viewModelScope.launch {
            val currentConfig = configRepository.getConfig()
            val newConfig = currentConfig.copy(
                minLatencyNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_LATENCY_MIN, currentConfig.minLatencyNs
                ), maxLatencyNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_LATENCY_MAX, currentConfig.maxLatencyNs
                ), minGranularityNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_GRANULARITY_MIN, currentConfig.minGranularityNs
                ), maxGranularityNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_GRANULARITY_MAX, currentConfig.maxGranularityNs
                ), minWakeupNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_WAKEUP_MIN, currentConfig.minWakeupNs
                ), maxWakeupNs = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_WAKEUP_MAX, currentConfig.maxWakeupNs
                ), minMigrationCost = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_MIGRATION_COST_MIN, currentConfig.minMigrationCost
                ), maxMigrationCost = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_MIGRATION_COST_MAX, currentConfig.maxMigrationCost
                ), minWaltInitPct = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_WALT_INIT_MIN, currentConfig.minWaltInitPct
                ), maxWaltInitPct = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_WALT_INIT_MAX, currentConfig.maxWaltInitPct
                ), minUclampMin = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_UCLAMP_MIN_MIN, currentConfig.minUclampMin
                ), maxUclampMin = cpuValues.getUpdatedLong(
                    ConfigKeys.CPU_UCLAMP_MIN_MAX, currentConfig.maxUclampMin
                )
            )
            appPreferencesRepository.setCachedCpuLimits(newConfig.toCpuMap().encodeToString())
            val success = configRepository.updateConfig(newConfig)
            if (success) {
                appPreferencesRepository.setNeedsReboot(true)
                _state.update { it.copy(config = newConfig) }
            }
        }
    }

    private fun applyStorageConfig(storageValues: Map<String, String>) {
        if (storageValues.isEmpty()) return
        viewModelScope.launch {
            val currentConfig = configRepository.getConfig()
            val newConfig = currentConfig.copy(
                minReadAhead = storageValues.getUpdatedLong(
                    ConfigKeys.STORAGE_READ_AHEAD_MIN, currentConfig.minReadAhead
                ), maxReadAhead = storageValues.getUpdatedLong(
                    ConfigKeys.STORAGE_READ_AHEAD_MAX, currentConfig.maxReadAhead
                ), minNrRequests = storageValues.getUpdatedLong(
                    ConfigKeys.STORAGE_NR_REQUESTS_MIN, currentConfig.minNrRequests
                ), maxNrRequests = storageValues.getUpdatedLong(
                    ConfigKeys.STORAGE_NR_REQUESTS_MAX, currentConfig.maxNrRequests
                )
            )
            appPreferencesRepository.setCachedStorageLimits(
                newConfig.toStorageMap().encodeToString()
            )
            val success = configRepository.updateConfig(newConfig)
            if (success) {
                appPreferencesRepository.setNeedsReboot(true)
                _state.update { it.copy(config = newConfig) }
            }
        }
    }

    private suspend fun refreshInternal() {
        if (!daemonRepository.checkDaemonExists()) {
            _state.update { it.copy(isDaemonMissing = true, isConfigMissing = false) }
            return
        }
        if (!configRepository.checkConfigExists()) {
            _state.update { it.copy(isDaemonMissing = false, isConfigMissing = true) }
            return
        }

        val config = configRepository.getConfig()
        val isCpuEnabled =
            config.minLatencyNs != null || config.maxLatencyNs != null || config.minGranularityNs != null || config.maxGranularityNs != null || config.minWakeupNs != null || config.maxWakeupNs != null || config.minMigrationCost != null || config.maxMigrationCost != null || config.minWaltInitPct != null || config.maxWaltInitPct != null || config.minUclampMin != null || config.maxUclampMin != null
        val isStorageEnabled =
            config.minReadAhead != null || config.maxReadAhead != null || config.minNrRequests != null || config.maxNrRequests != null

        _state.update {
            it.copy(
                isDaemonMissing = false,
                isConfigMissing = false,
                config = config,
                cpuLimitsEnabled = isCpuEnabled,
                storageLimitsEnabled = isStorageEnabled
            )
        }
    }

    private fun Map<String, String>.encodeToString(): String =
        entries.joinToString(";") { "${it.key}=${it.value}" }

    private fun String.decodeToMap(): Map<String, String> {
        if (this.isBlank()) return emptyMap()
        return splitToSequence(';').mapNotNull { pair ->
            val index = pair.indexOf('=')
            if (index != -1) pair.substring(0, index) to pair.substring(index + 1) else null
        }.toMap()
    }

    private fun QosConfig.toCpuMap(): Map<String, String> = mapOf(
        ConfigKeys.CPU_LATENCY_MIN to (minLatencyNs?.toString() ?: ""),
        ConfigKeys.CPU_LATENCY_MAX to (maxLatencyNs?.toString() ?: ""),
        ConfigKeys.CPU_GRANULARITY_MIN to (minGranularityNs?.toString() ?: ""),
        ConfigKeys.CPU_GRANULARITY_MAX to (maxGranularityNs?.toString() ?: ""),
        ConfigKeys.CPU_WAKEUP_MIN to (minWakeupNs?.toString() ?: ""),
        ConfigKeys.CPU_WAKEUP_MAX to (maxWakeupNs?.toString() ?: ""),
        ConfigKeys.CPU_MIGRATION_COST_MIN to (minMigrationCost?.toString() ?: ""),
        ConfigKeys.CPU_MIGRATION_COST_MAX to (maxMigrationCost?.toString() ?: ""),
        ConfigKeys.CPU_WALT_INIT_MIN to (minWaltInitPct?.toString() ?: ""),
        ConfigKeys.CPU_WALT_INIT_MAX to (maxWaltInitPct?.toString() ?: ""),
        ConfigKeys.CPU_UCLAMP_MIN_MIN to (minUclampMin?.toString() ?: ""),
        ConfigKeys.CPU_UCLAMP_MIN_MAX to (maxUclampMin?.toString() ?: "")
    )

    private fun QosConfig.toStorageMap(): Map<String, String> = mapOf(
        ConfigKeys.STORAGE_READ_AHEAD_MIN to (minReadAhead?.toString() ?: ""),
        ConfigKeys.STORAGE_READ_AHEAD_MAX to (maxReadAhead?.toString() ?: ""),
        ConfigKeys.STORAGE_NR_REQUESTS_MIN to (minNrRequests?.toString() ?: ""),
        ConfigKeys.STORAGE_NR_REQUESTS_MAX to (maxNrRequests?.toString() ?: "")
    )
}