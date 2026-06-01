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
import androidx.compose.material3.ExperimentalMaterial3Api
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
import com.seclususs.qos.ui.components.cards.QosIconTitleCard
import com.seclususs.qos.ui.components.inputs.AnimatedSwitch
import com.seclususs.qos.ui.components.layout.BottomSheet
import com.seclususs.qos.ui.components.layout.QosScreen

@Composable
fun ModulesScreen(viewModel: ModulesViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    QosScreen(
        title = stringResource(id = R.string.nav_modules),
        isDaemonMissing = state.isDaemonMissing,
        isConfigMissing = state.isConfigMissing
    ) {
        ModulesContent(state, viewModel)
    }

    if (state.selectedModuleForDetails != null) {
        ModuleDetailsSheet(
            moduleType = state.selectedModuleForDetails!!,
            onDismiss = { viewModel.onEvent(ModulesEvent.DismissModuleDetails) })
    }
}

@Composable
private fun ModulesContent(state: ModulesState, viewModel: ModulesViewModel) {
    val modulesList = listOf(
        Triple(ModuleType.BLOCKER, R.string.module_blocker_title, R.string.module_blocker_subtitle),
        Triple(ModuleType.CLEANER, R.string.module_cleaner_title, R.string.module_cleaner_subtitle),
        Triple(ModuleType.CPU, R.string.module_cpu_title, R.string.module_cpu_subtitle),
        Triple(ModuleType.STORAGE, R.string.module_storage_title, R.string.module_storage_subtitle),
        Triple(ModuleType.TWEAKS, R.string.module_tweaks_title, R.string.module_tweaks_subtitle)
    )

    Column(
        modifier = Modifier.fillMaxWidth().verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        modulesList.forEach { (type, titleRes, subtitleRes) ->
            val isChecked = isModuleEnabled(state.config, type)
            val isProcessing = state.processingModules.contains(type)
            QosIconTitleCard(
                title = stringResource(id = titleRes),
                subtitle = stringResource(id = subtitleRes),
                icon = getIconForModule(type),
                onClick = {
                    if (!isProcessing) viewModel.onEvent(
                        ModulesEvent.ToggleModule(
                            type, !isChecked
                        )
                    )
                },
                onLongClick = { viewModel.onEvent(ModulesEvent.ShowModuleDetails(type)) },
                trailingContent = {
                    AnimatedSwitch(
                        isChecked = isChecked, isProcessing = isProcessing
                    )
                })
        }
        Spacer(modifier = Modifier.height(80.dp))
    }
}

private fun getIconForModule(type: ModuleType): ImageVector = when (type) {
    ModuleType.BLOCKER -> Icons.Filled.Security
    ModuleType.CLEANER -> Icons.Filled.CleaningServices
    ModuleType.CPU -> Icons.Filled.Memory
    ModuleType.STORAGE -> Icons.Filled.Storage
    ModuleType.TWEAKS -> Icons.Filled.Build
}

private fun isModuleEnabled(config: QosConfig, type: ModuleType): Boolean = when (type) {
    ModuleType.BLOCKER -> config.blockerEnabled
    ModuleType.CLEANER -> config.cleanerEnabled
    ModuleType.CPU -> config.cpuEnabled
    ModuleType.STORAGE -> config.storageEnabled
    ModuleType.TWEAKS -> config.tweaksEnabled
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ModuleDetailsSheet(moduleType: ModuleType, onDismiss: () -> Unit) {
    val (titleRes, descRes) = when (moduleType) {
        ModuleType.BLOCKER -> Pair(R.string.module_blocker_title, R.string.module_blocker_desc)
        ModuleType.CLEANER -> Pair(R.string.module_cleaner_title, R.string.module_cleaner_desc)
        ModuleType.CPU -> Pair(R.string.module_cpu_title, R.string.module_cpu_desc)
        ModuleType.STORAGE -> Pair(R.string.module_storage_title, R.string.module_storage_desc)
        ModuleType.TWEAKS -> Pair(R.string.module_tweaks_title, R.string.module_tweaks_desc)
    }
    BottomSheet(title = stringResource(id = titleRes), onDismissRequest = onDismiss) {
        Text(
            text = stringResource(id = descRes),
            style = MaterialTheme.typography.bodyLarge.copy(lineHeight = 22.sp),
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
        )
        Spacer(modifier = Modifier.height(24.dp))
    }
}