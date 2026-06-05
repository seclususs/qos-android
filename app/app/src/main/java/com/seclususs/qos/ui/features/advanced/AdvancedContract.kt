package com.seclususs.qos.ui.features.advanced

import androidx.compose.runtime.Stable
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.model.SystemStatus

enum class AdvancedSheetType { CPU, STORAGE }

@Stable
data class AdvancedState(
    val systemStatus: SystemStatus = SystemStatus.OK,
    val config: QosConfig = QosConfig(),
    val cpuLimitsEnabled: Boolean = false,
    val storageLimitsEnabled: Boolean = false,
    val cachedCpuValues: Map<String, String> = emptyMap(),
    val cachedStorageValues: Map<String, String> = emptyMap(),
    val isProcessingCpuToggle: Boolean = false,
    val isProcessingStorageToggle: Boolean = false,
    val activeSheet: AdvancedSheetType? = null
)

sealed interface AdvancedEvent {
    data class ToggleCpu(val enabled: Boolean) : AdvancedEvent
    data class ToggleStorage(val enabled: Boolean) : AdvancedEvent
    data class ShowSheet(val type: AdvancedSheetType) : AdvancedEvent
    data class SaveAndHideSheet(
        val cpuValues: Map<String, String>? = null, val storageValues: Map<String, String>? = null
    ) : AdvancedEvent

    data object RefreshStatus : AdvancedEvent
}