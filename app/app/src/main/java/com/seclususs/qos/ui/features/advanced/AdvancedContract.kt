package com.seclususs.qos.ui.features.advanced

import com.seclususs.qos.domain.model.QosConfig

enum class ApplyState { IDLE, LOADING, SUCCESS }

enum class AdvancedSheetType { CPU, STORAGE }

data class AdvancedState(
    val isDaemonMissing: Boolean = false,
    val isConfigMissing: Boolean = false,
    val config: QosConfig = QosConfig(),
    val cpuLimitsEnabled: Boolean = false,
    val storageLimitsEnabled: Boolean = false,
    val cpuValues: Map<String, String> = emptyMap(),
    val storageValues: Map<String, String> = emptyMap(),
    val cachedCpuValues: Map<String, String> = emptyMap(),
    val cachedStorageValues: Map<String, String> = emptyMap(),
    val isProcessingCpuToggle: Boolean = false,
    val isProcessingStorageToggle: Boolean = false,
    val cpuApplyState: ApplyState = ApplyState.IDLE,
    val storageApplyState: ApplyState = ApplyState.IDLE,
    val activeSheet: AdvancedSheetType? = null,
    val snackbarMessageResId: Int? = null,
    val snackbarIsError: Boolean = false,
    val snackbarVisible: Boolean = false
)

sealed interface AdvancedEvent {
    data class ToggleCpu(val enabled: Boolean) : AdvancedEvent
    data class ToggleStorage(val enabled: Boolean) : AdvancedEvent
    data class ShowSheet(val type: AdvancedSheetType) : AdvancedEvent
    object HideSheet : AdvancedEvent
    data class ApplyCpuConfig(val cpuValues: Map<String, String>) : AdvancedEvent
    data class ApplyStorageConfig(val storageValues: Map<String, String>) : AdvancedEvent
    object RefreshStatus : AdvancedEvent
    object DismissSnackbar : AdvancedEvent
}