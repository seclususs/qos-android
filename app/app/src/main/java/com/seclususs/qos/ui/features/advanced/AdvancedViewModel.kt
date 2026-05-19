package com.seclususs.qos.ui.features.advanced

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.R
import com.seclususs.qos.data.local.AppStore
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.usecase.CheckConfigUseCase
import com.seclususs.qos.domain.usecase.DaemonStatusUseCase
import com.seclususs.qos.domain.usecase.GetConfigUseCase
import com.seclususs.qos.domain.usecase.ToggleDaemonUseCase
import com.seclususs.qos.domain.usecase.UpdateConfigUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AdvancedViewModel @Inject constructor(
    private val getConfigUseCase: GetConfigUseCase,
    private val updateConfigUseCase: UpdateConfigUseCase,
    private val toggleDaemonUseCase: ToggleDaemonUseCase,
    private val daemonStatusUseCase: DaemonStatusUseCase,
    private val checkConfigExistsUseCase: CheckConfigUseCase,
    private val appStore: AppStore
) : ViewModel() {

    private val _state = MutableStateFlow(AdvancedState())
    val state: StateFlow<AdvancedState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            appStore.cachedCpuLimitsFlow.collect { encoded ->
                _state.update { it.copy(cachedCpuValues = encoded.decodeToMap()) }
            }
        }
        viewModelScope.launch {
            appStore.cachedStorageLimitsFlow.collect { encoded ->
                _state.update { it.copy(cachedStorageValues = encoded.decodeToMap()) }
            }
        }
        viewModelScope.launch { refreshInternal() }
    }

    fun onEvent(event: AdvancedEvent) {
        when (event) {
            is AdvancedEvent.ToggleCpu -> handleToggleCpu(event.enabled)
            is AdvancedEvent.ToggleStorage -> handleToggleStorage(event.enabled)
            is AdvancedEvent.UpdateCpuField -> {
                _state.update {
                    it.copy(
                        cpuValues = it.cpuValues.toMutableMap()
                            .apply { put(event.key, event.value) })
                }
            }

            is AdvancedEvent.UpdateStorageField -> {
                _state.update {
                    it.copy(
                        storageValues = it.storageValues.toMutableMap()
                            .apply { put(event.key, event.value) })
                }
            }

            is AdvancedEvent.ShowSheet -> _state.update { it.copy(activeSheet = event.type) }
            is AdvancedEvent.HideSheet -> _state.update {
                it.copy(
                    activeSheet = null,
                    cpuApplyState = ApplyState.IDLE,
                    storageApplyState = ApplyState.IDLE
                )
            }

            is AdvancedEvent.ApplyCpuConfig -> applyCpuConfig()
            is AdvancedEvent.ApplyStorageConfig -> applyStorageConfig()
            is AdvancedEvent.RefreshStatus -> viewModelScope.launch { refreshInternal() }
            is AdvancedEvent.DismissSnackbar -> _state.update { it.copy(snackbarVisible = false) }
        }
    }

    private fun handleToggleCpu(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingCpuToggle = true) }
            delay(400)

            val currentConfig = _state.value.config
            val newConfig: QosConfig

            if (enabled) {
                val cached = _state.value.cachedCpuValues
                val default = QosConfig()
                newConfig = currentConfig.copy(
                    minLatencyNs = getCachedOrDef(cached, "latency_min", default.minLatencyNs),
                    maxLatencyNs = getCachedOrDef(cached, "latency_max", default.maxLatencyNs),
                    minGranularityNs = getCachedOrDef(
                        cached, "granularity_min", default.minGranularityNs
                    ),
                    maxGranularityNs = getCachedOrDef(
                        cached, "granularity_max", default.maxGranularityNs
                    ),
                    minWakeupNs = getCachedOrDef(cached, "wakeup_min", default.minWakeupNs),
                    maxWakeupNs = getCachedOrDef(cached, "wakeup_max", default.maxWakeupNs),
                    minMigrationCost = getCachedOrDef(
                        cached, "migration_cost_min", default.minMigrationCost
                    ),
                    maxMigrationCost = getCachedOrDef(
                        cached, "migration_cost_max", default.maxMigrationCost
                    ),
                    minWaltInitPct = getCachedOrDef(
                        cached, "walt_init_min", default.minWaltInitPct
                    ),
                    maxWaltInitPct = getCachedOrDef(
                        cached, "walt_init_max", default.maxWaltInitPct
                    ),
                    minUclampMin = getCachedOrDef(cached, "uclamp_min_min", default.minUclampMin),
                    maxUclampMin = getCachedOrDef(cached, "uclamp_min_max", default.maxUclampMin)
                )
            } else {
                appStore.setCachedCpuLimits(_state.value.cpuValues.encodeToString())
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

            saveAndRestartDaemon(newConfig) { success ->
                if (success) {
                    _state.update {
                        it.copy(
                            config = newConfig,
                            cpuLimitsEnabled = enabled,
                            cpuValues = newConfig.toCpuMap()
                        )
                    }
                }
            }
            _state.update { it.copy(isProcessingCpuToggle = false) }
        }
    }

    private fun handleToggleStorage(enabled: Boolean) {
        viewModelScope.launch {
            _state.update { it.copy(isProcessingStorageToggle = true) }
            delay(400)

            val currentConfig = _state.value.config
            val newConfig: QosConfig

            if (enabled) {
                val cached = _state.value.cachedStorageValues
                val default = QosConfig()
                newConfig = currentConfig.copy(
                    minReadAhead = getCachedOrDef(cached, "read_ahead_min", default.minReadAhead),
                    maxReadAhead = getCachedOrDef(cached, "read_ahead_max", default.maxReadAhead),
                    minNrRequests = getCachedOrDef(
                        cached, "nr_requests_min", default.minNrRequests
                    ),
                    maxNrRequests = getCachedOrDef(cached, "nr_requests_max", default.maxNrRequests)
                )
            } else {
                appStore.setCachedStorageLimits(_state.value.storageValues.encodeToString())
                newConfig = currentConfig.copy(
                    minReadAhead = null,
                    maxReadAhead = null,
                    minNrRequests = null,
                    maxNrRequests = null
                )
            }

            saveAndRestartDaemon(newConfig) { success ->
                if (success) {
                    _state.update {
                        it.copy(
                            config = newConfig,
                            storageLimitsEnabled = enabled,
                            storageValues = newConfig.toStorageMap()
                        )
                    }
                }
            }
            _state.update { it.copy(isProcessingStorageToggle = false) }
        }
    }

    private fun applyCpuConfig() {
        viewModelScope.launch {
            _state.update { it.copy(cpuApplyState = ApplyState.LOADING) }
            val currentConfig = _state.value.config
            val cpuValues = _state.value.cpuValues

            appStore.setCachedCpuLimits(cpuValues.encodeToString())

            val newConfig = currentConfig.copy(
                minLatencyNs = cpuValues["latency_min"]?.toLongOrNull(),
                maxLatencyNs = cpuValues["latency_max"]?.toLongOrNull(),
                minGranularityNs = cpuValues["granularity_min"]?.toLongOrNull(),
                maxGranularityNs = cpuValues["granularity_max"]?.toLongOrNull(),
                minWakeupNs = cpuValues["wakeup_min"]?.toLongOrNull(),
                maxWakeupNs = cpuValues["wakeup_max"]?.toLongOrNull(),
                minMigrationCost = cpuValues["migration_cost_min"]?.toLongOrNull(),
                maxMigrationCost = cpuValues["migration_cost_max"]?.toLongOrNull(),
                minWaltInitPct = cpuValues["walt_init_min"]?.toLongOrNull(),
                maxWaltInitPct = cpuValues["walt_init_max"]?.toLongOrNull(),
                minUclampMin = cpuValues["uclamp_min_min"]?.toLongOrNull(),
                maxUclampMin = cpuValues["uclamp_min_max"]?.toLongOrNull()
            )

            val success = updateConfigUseCase(newConfig)
            if (success) {
                toggleDaemonUseCase.restart()
                _state.update {
                    it.copy(
                        config = newConfig,
                        cpuApplyState = ApplyState.SUCCESS,
                        snackbarMessageResId = R.string.module_update_success,
                        snackbarIsError = false,
                        snackbarVisible = true
                    )
                }
            } else {
                _state.update {
                    it.copy(
                        cpuApplyState = ApplyState.IDLE,
                        snackbarMessageResId = R.string.module_update_error,
                        snackbarIsError = true,
                        snackbarVisible = true
                    )
                }
            }

            delay(2000)
            _state.update { it.copy(snackbarVisible = false) }
        }
    }

    private fun applyStorageConfig() {
        viewModelScope.launch {
            _state.update { it.copy(storageApplyState = ApplyState.LOADING) }
            val currentConfig = _state.value.config
            val storageValues = _state.value.storageValues

            appStore.setCachedStorageLimits(storageValues.encodeToString())

            val newConfig = currentConfig.copy(
                minReadAhead = storageValues["read_ahead_min"]?.toLongOrNull(),
                maxReadAhead = storageValues["read_ahead_max"]?.toLongOrNull(),
                minNrRequests = storageValues["nr_requests_min"]?.toLongOrNull(),
                maxNrRequests = storageValues["nr_requests_max"]?.toLongOrNull()
            )

            val success = updateConfigUseCase(newConfig)
            if (success) {
                toggleDaemonUseCase.restart()
                _state.update {
                    it.copy(
                        config = newConfig,
                        storageApplyState = ApplyState.SUCCESS,
                        snackbarMessageResId = R.string.module_update_success,
                        snackbarIsError = false,
                        snackbarVisible = true
                    )
                }
            } else {
                _state.update {
                    it.copy(
                        storageApplyState = ApplyState.IDLE,
                        snackbarMessageResId = R.string.module_update_error,
                        snackbarIsError = true,
                        snackbarVisible = true
                    )
                }
            }

            delay(2000)
            _state.update { it.copy(snackbarVisible = false) }
        }
    }

    private suspend fun saveAndRestartDaemon(newConfig: QosConfig, updateState: (Boolean) -> Unit) {
        val success = updateConfigUseCase(newConfig)
        if (success) {
            toggleDaemonUseCase.restart()
            updateState(true)
            _state.update {
                it.copy(
                    snackbarMessageResId = R.string.module_update_success,
                    snackbarIsError = false,
                    snackbarVisible = true
                )
            }
        } else {
            updateState(false)
            _state.update {
                it.copy(
                    snackbarMessageResId = R.string.module_update_error,
                    snackbarIsError = true,
                    snackbarVisible = true
                )
            }
        }
        delay(3000)
        _state.update { it.copy(snackbarVisible = false) }
    }

    private suspend fun refreshInternal() {
        if (!daemonStatusUseCase.checkDaemonExists()) {
            _state.update { it.copy(isDaemonMissing = true, isConfigMissing = false) }
            return
        }
        if (!checkConfigExistsUseCase()) {
            _state.update { it.copy(isDaemonMissing = false, isConfigMissing = true) }
            return
        }

        val config = getConfigUseCase()
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
                storageLimitsEnabled = isStorageEnabled,
                cpuValues = config.toCpuMap(),
                storageValues = config.toStorageMap()
            )
        }
    }

    private fun getCachedOrDef(cached: Map<String, String>, key: String, def: Long?): Long? {
        val str = cached[key]
        return if (str != null) str.toLongOrNull() else def
    }

    private fun Map<String, String>.encodeToString(): String =
        entries.joinToString(";") { "${it.key}=${it.value}" }

    private fun String.decodeToMap(): Map<String, String> {
        if (this.isBlank()) return emptyMap()
        return split(";").mapNotNull {
            val parts = it.split("=")
            if (parts.size == 2) parts[0] to parts[1] else null
        }.toMap()
    }

    private fun QosConfig.toCpuMap(): Map<String, String> = mapOf(
        "latency_min" to (minLatencyNs?.toString() ?: ""),
        "latency_max" to (maxLatencyNs?.toString() ?: ""),
        "granularity_min" to (minGranularityNs?.toString() ?: ""),
        "granularity_max" to (maxGranularityNs?.toString() ?: ""),
        "wakeup_min" to (minWakeupNs?.toString() ?: ""),
        "wakeup_max" to (maxWakeupNs?.toString() ?: ""),
        "migration_cost_min" to (minMigrationCost?.toString() ?: ""),
        "migration_cost_max" to (maxMigrationCost?.toString() ?: ""),
        "walt_init_min" to (minWaltInitPct?.toString() ?: ""),
        "walt_init_max" to (maxWaltInitPct?.toString() ?: ""),
        "uclamp_min_min" to (minUclampMin?.toString() ?: ""),
        "uclamp_min_max" to (maxUclampMin?.toString() ?: "")
    )

    private fun QosConfig.toStorageMap(): Map<String, String> = mapOf(
        "read_ahead_min" to (minReadAhead?.toString() ?: ""),
        "read_ahead_max" to (maxReadAhead?.toString() ?: ""),
        "nr_requests_min" to (minNrRequests?.toString() ?: ""),
        "nr_requests_max" to (maxNrRequests?.toString() ?: "")
    )
}