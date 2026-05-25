package com.seclususs.qos.ui.features.modules

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.CleaningServices
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.seclususs.qos.R
import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.ui.components.AnimatedSwitch
import com.seclususs.qos.ui.components.BottomSheet
import com.seclususs.qos.ui.components.QosCard
import com.seclususs.qos.ui.components.QosTopSnackbar
import com.seclususs.qos.ui.components.StateAwareContent

@Composable
fun ModulesScreen(
    viewModel: ModulesViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .padding(horizontal = 24.dp)
                .padding(top = 2.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            Text(
                text = stringResource(id = R.string.nav_modules),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            StateAwareContent(
                isDaemonMissing = state.isDaemonMissing, isConfigMissing = state.isConfigMissing
            ) {
                ModulesContent(state, viewModel)
            }
        }

        QosTopSnackbar(
            messageResId = state.snackbarMessageResId,
            isError = state.snackbarIsError,
            isVisible = state.snackbarVisible,
            modifier = Modifier.align(Alignment.TopCenter)
        )

        if (state.selectedModuleForDetails != null) {
            ModuleDetailsSheet(
                moduleType = state.selectedModuleForDetails!!,
                onDismiss = { viewModel.onEvent(ModulesEvent.DismissModuleDetails) })
        }
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
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        modulesList.forEach { (type, titleRes, subtitleRes) ->
            ModuleItem(
                title = stringResource(id = titleRes),
                subtitle = stringResource(id = subtitleRes),
                icon = getIconForModule(type),
                isChecked = isModuleEnabled(state.config, type),
                isProcessing = state.processingModules.contains(type),
                onToggle = { viewModel.onEvent(ModulesEvent.ToggleModule(type, it)) },
                onLongPress = { viewModel.onEvent(ModulesEvent.ShowModuleDetails(type)) })
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

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun ModuleItem(
    title: String,
    subtitle: String,
    icon: ImageVector,
    isChecked: Boolean,
    isProcessing: Boolean,
    onToggle: (Boolean) -> Unit,
    onLongPress: () -> Unit
) {
    QosCard(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .combinedClickable(
                    onClick = { if (!isProcessing) onToggle(!isChecked) }, onLongClick = onLongPress
                )
                .padding(16.dp), verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(24.dp)
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.onSurface
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.labelLarge.copy(fontSize = 12.sp),
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f),
                    maxLines = 2
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            AnimatedSwitch(isChecked = isChecked, isProcessing = isProcessing)
        }
    }
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
    BottomSheet(onDismissRequest = onDismiss) {
        Text(
            text = stringResource(id = titleRes),
            style = MaterialTheme.typography.titleLarge.copy(fontSize = 20.sp),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(bottom = 12.dp)
        )
        Text(
            text = stringResource(id = descRes),
            style = MaterialTheme.typography.bodyLarge.copy(lineHeight = 22.sp),
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
        )
        Spacer(modifier = Modifier.height(24.dp))
    }
}