package com.seclususs.qos.ui.features.modules

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.CleaningServices
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.seclususs.qos.R
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.ui.components.cards.ToggleCard
import com.seclususs.qos.ui.components.layout.QosScreen

private data class ModuleItem(
    val type: ModuleType, val titleRes: Int, val subtitleRes: Int, val icon: ImageVector
)

@Composable
fun ModulesScreen(viewModel: ModulesViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    QosScreen(
        title = stringResource(id = R.string.nav_modules),
        isDaemonMissing = state.isDaemonMissing,
        isConfigMissing = state.isConfigMissing
    ) {
        ModulesContent(state, onEvent = viewModel::onEvent)
    }
}

@Composable
private fun ModulesContent(state: ModulesState, onEvent: (ModulesEvent) -> Unit) {
    val modulesList = listOf(
        ModuleItem(
            ModuleType.BLOCKER,
            R.string.module_blocker_title,
            R.string.module_blocker_subtitle,
            Icons.Filled.Security
        ), ModuleItem(
            ModuleType.CLEANER,
            R.string.module_cleaner_title,
            R.string.module_cleaner_subtitle,
            Icons.Filled.CleaningServices
        ), ModuleItem(
            ModuleType.CPU,
            R.string.module_cpu_title,
            R.string.module_cpu_subtitle,
            Icons.Filled.Memory
        ), ModuleItem(
            ModuleType.STORAGE,
            R.string.module_storage_title,
            R.string.module_storage_subtitle,
            Icons.Filled.Storage
        ), ModuleItem(
            ModuleType.TWEAKS,
            R.string.module_tweaks_title,
            R.string.module_tweaks_subtitle,
            Icons.Filled.Build
        )
    )

    Column(
        modifier = Modifier.fillMaxWidth().verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        modulesList.forEach { item ->
            val isChecked = isModuleEnabled(state.config, item.type)
            val isProcessing = state.processingModules.contains(item.type)
            val isExpanded = state.expandedModule == item.type
            ToggleCard(
                title = stringResource(id = item.titleRes),
                subtitle = stringResource(id = item.subtitleRes),
                icon = item.icon,
                isToggled = isChecked,
                isProcessing = isProcessing,
                onToggle = { onEvent(ModulesEvent.ToggleModule(item.type, it)) },
                onClick = { onEvent(ModulesEvent.ToggleModuleExpansion(item.type)) },
                isExpanded = isExpanded
            ) {
                val descRes = when (item.type) {
                    ModuleType.BLOCKER -> R.string.module_blocker_desc
                    ModuleType.CLEANER -> R.string.module_cleaner_desc
                    ModuleType.CPU -> R.string.module_cpu_desc
                    ModuleType.STORAGE -> R.string.module_storage_desc
                    ModuleType.TWEAKS -> R.string.module_tweaks_desc
                }
                Text(
                    text = stringResource(id = descRes),
                    style = MaterialTheme.typography.bodyLarge.copy(lineHeight = 22.sp),
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
                )
            }
        }
        Spacer(modifier = Modifier.height(80.dp))
    }
}

private fun isModuleEnabled(config: QosConfig, type: ModuleType): Boolean = when (type) {
    ModuleType.BLOCKER -> config.blockerEnabled
    ModuleType.CLEANER -> config.cleanerEnabled
    ModuleType.CPU -> config.cpuEnabled
    ModuleType.STORAGE -> config.storageEnabled
    ModuleType.TWEAKS -> config.tweaksEnabled
}