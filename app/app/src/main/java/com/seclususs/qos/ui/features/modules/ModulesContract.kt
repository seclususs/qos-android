package com.seclususs.qos.ui.features.modules

import androidx.compose.runtime.Stable
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.model.SystemStatus

enum class ModuleType {
    BLOCKER, CLEANER, CPU, STORAGE, TWEAKS
}

@Stable
data class ModulesState(
    val systemStatus: SystemStatus = SystemStatus.OK,
    val config: QosConfig = QosConfig(),
    val processingModules: Set<ModuleType> = emptySet(),
    val expandedModule: ModuleType? = null
)

sealed interface ModulesEvent {
    data class ToggleModule(val type: ModuleType, val enabled: Boolean) : ModulesEvent
    data class ToggleModuleExpansion(val type: ModuleType) : ModulesEvent
    data object RefreshStatus : ModulesEvent
}