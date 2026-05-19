package com.seclususs.qos.ui.features.modules

import com.seclususs.qos.domain.model.QosConfig

enum class ModuleType {
    BLOCKER, CLEANER, CPU, STORAGE, TWEAKS
}

data class ModulesState(
    val isDaemonMissing: Boolean = false,
    val isConfigMissing: Boolean = false,
    val config: QosConfig = QosConfig(),
    val processingModules: Set<ModuleType> = emptySet(),
    val selectedModuleForDetails: ModuleType? = null,
    val snackbarMessageResId: Int? = null,
    val snackbarIsError: Boolean = false,
    val snackbarVisible: Boolean = false
)

sealed interface ModulesEvent {
    data class ToggleModule(val type: ModuleType, val enabled: Boolean) : ModulesEvent
    data class ShowModuleDetails(val type: ModuleType) : ModulesEvent
    object DismissModuleDetails : ModulesEvent
    object RefreshStatus : ModulesEvent
    object DismissSnackbar : ModulesEvent
}