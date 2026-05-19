package com.seclususs.qos.ui.features.modules

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
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
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.CleaningServices
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.seclususs.qos.R
import com.seclususs.qos.ui.components.BottomSheet
import com.seclususs.qos.ui.components.QosCard
import com.seclususs.qos.ui.components.TopSnackbar

@Composable
fun ModulesScreen(
    viewModel: ModulesViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsState()

    val targetUiState = when {
        state.isDaemonMissing -> 1
        state.isConfigMissing -> 2
        else -> 0
    }

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

            AnimatedContent(
                targetState = targetUiState, transitionSpec = {
                    (fadeIn(tween(300)) + scaleIn(
                        tween(300), initialScale = 0.9f
                    )) togetherWith (fadeOut(tween(200)) + scaleOut(
                        tween(200), targetScale = 0.9f
                    )) using SizeTransform(clip = false) { _, _ ->
                        spring(
                            dampingRatio = Spring.DampingRatioLowBouncy,
                            stiffness = Spring.StiffnessLow
                        )
                    }
                }, label = "modules_ui_state_transition", modifier = Modifier.fillMaxWidth()
            ) { uiState ->
                when (uiState) {
                    1 -> {
                        MissingDaemonCard(onRefresh = { viewModel.onEvent(ModulesEvent.RefreshStatus) })
                    }

                    2 -> {
                        MissingConfigCard(onRefresh = { viewModel.onEvent(ModulesEvent.RefreshStatus) })
                    }

                    else -> {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .verticalScroll(rememberScrollState()),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            ModuleItem(
                                title = stringResource(id = R.string.module_blocker_title),
                                subtitle = stringResource(id = R.string.module_blocker_subtitle),
                                icon = Icons.Filled.Security,
                                isChecked = state.config.blockerEnabled,
                                isProcessing = state.processingModules.contains(ModuleType.BLOCKER),
                                onToggle = {
                                    viewModel.onEvent(
                                        ModulesEvent.ToggleModule(
                                            ModuleType.BLOCKER, it
                                        )
                                    )
                                },
                                onLongPress = {
                                    viewModel.onEvent(
                                        ModulesEvent.ShowModuleDetails(
                                            ModuleType.BLOCKER
                                        )
                                    )
                                })

                            ModuleItem(
                                title = stringResource(id = R.string.module_cleaner_title),
                                subtitle = stringResource(id = R.string.module_cleaner_subtitle),
                                icon = Icons.Filled.CleaningServices,
                                isChecked = state.config.cleanerEnabled,
                                isProcessing = state.processingModules.contains(ModuleType.CLEANER),
                                onToggle = {
                                    viewModel.onEvent(
                                        ModulesEvent.ToggleModule(
                                            ModuleType.CLEANER, it
                                        )
                                    )
                                },
                                onLongPress = {
                                    viewModel.onEvent(
                                        ModulesEvent.ShowModuleDetails(
                                            ModuleType.CLEANER
                                        )
                                    )
                                })

                            ModuleItem(
                                title = stringResource(id = R.string.module_cpu_title),
                                subtitle = stringResource(id = R.string.module_cpu_subtitle),
                                icon = Icons.Filled.Memory,
                                isChecked = state.config.cpuEnabled,
                                isProcessing = state.processingModules.contains(ModuleType.CPU),
                                onToggle = {
                                    viewModel.onEvent(
                                        ModulesEvent.ToggleModule(
                                            ModuleType.CPU, it
                                        )
                                    )
                                },
                                onLongPress = {
                                    viewModel.onEvent(
                                        ModulesEvent.ShowModuleDetails(
                                            ModuleType.CPU
                                        )
                                    )
                                })

                            ModuleItem(
                                title = stringResource(id = R.string.module_storage_title),
                                subtitle = stringResource(id = R.string.module_storage_subtitle),
                                icon = Icons.Filled.Storage,
                                isChecked = state.config.storageEnabled,
                                isProcessing = state.processingModules.contains(ModuleType.STORAGE),
                                onToggle = {
                                    viewModel.onEvent(
                                        ModulesEvent.ToggleModule(
                                            ModuleType.STORAGE, it
                                        )
                                    )
                                },
                                onLongPress = {
                                    viewModel.onEvent(
                                        ModulesEvent.ShowModuleDetails(
                                            ModuleType.STORAGE
                                        )
                                    )
                                })

                            ModuleItem(
                                title = stringResource(id = R.string.module_tweaks_title),
                                subtitle = stringResource(id = R.string.module_tweaks_subtitle),
                                icon = Icons.Filled.Build,
                                isChecked = state.config.tweaksEnabled,
                                isProcessing = state.processingModules.contains(ModuleType.TWEAKS),
                                onToggle = {
                                    viewModel.onEvent(
                                        ModulesEvent.ToggleModule(
                                            ModuleType.TWEAKS, it
                                        )
                                    )
                                },
                                onLongPress = {
                                    viewModel.onEvent(
                                        ModulesEvent.ShowModuleDetails(
                                            ModuleType.TWEAKS
                                        )
                                    )
                                })

                            Spacer(modifier = Modifier.height(80.dp))
                        }
                    }
                }
            }
        }

        val snackbarMessage = state.snackbarMessageResId?.let { stringResource(id = it) } ?: ""
        val snackbarIcon =
            if (state.snackbarIsError) Icons.Filled.Error else Icons.Filled.CheckCircle
        TopSnackbar(
            message = snackbarMessage,
            isVisible = state.snackbarVisible,
            isError = state.snackbarIsError,
            icon = snackbarIcon,
            modifier = Modifier.align(Alignment.TopCenter)
        )

        if (state.selectedModuleForDetails != null) {
            ModuleDetailsSheet(
                moduleType = state.selectedModuleForDetails!!,
                onDismiss = { viewModel.onEvent(ModulesEvent.DismissModuleDetails) })
        }
    }
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

            Column(
                modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center
            ) {
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

            AnimatedQosSwitch(
                isChecked = isChecked, isProcessing = isProcessing
            )
        }
    }
}

@Composable
private fun AnimatedQosSwitch(
    isChecked: Boolean, isProcessing: Boolean
) {
    val switchWidth = 52.dp
    val switchHeight = 30.dp
    val thumbSize = 22.dp
    val thumbPadding = 4.dp

    val trackColor by animateColorAsState(
        targetValue = if (isChecked) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface.copy(
            alpha = 0.2f
        ), animationSpec = tween(300), label = "switchTrackColor"
    )

    val thumbOffset by animateDpAsState(
        targetValue = if (isChecked) (switchWidth - thumbSize - thumbPadding) else thumbPadding,
        animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
        label = "switchThumbOffset"
    )

    Box(
        modifier = Modifier
            .width(switchWidth)
            .height(switchHeight)
            .clip(CircleShape)
            .background(trackColor), contentAlignment = Alignment.CenterStart
    ) {
        Box(
            modifier = Modifier
                .offset { IntOffset(thumbOffset.roundToPx(), 0) }
                .size(thumbSize)
                .shadow(elevation = 2.dp, shape = CircleShape)
                .clip(CircleShape)
                .background(Color.White), contentAlignment = Alignment.Center) {
            AnimatedContent(
                targetState = isProcessing, transitionSpec = {
                    (fadeIn(tween(200)) + scaleIn(tween(200))) togetherWith (fadeOut(tween(200)) + scaleOut(
                        tween(200)
                    ))
                }, label = "switchIconAnimation"
            ) { processing ->
                if (processing) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(14.dp),
                        color = MaterialTheme.colorScheme.primary,
                        strokeWidth = 2.dp
                    )
                } else {
                    Icon(
                        imageVector = if (isChecked) Icons.Filled.Check else Icons.Filled.Close,
                        contentDescription = null,
                        tint = if (isChecked) MaterialTheme.colorScheme.primary else Color.Gray,
                        modifier = Modifier.size(14.dp)
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ModuleDetailsSheet(
    moduleType: ModuleType, onDismiss: () -> Unit
) {
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

@Composable
private fun MissingDaemonCard(onRefresh: () -> Unit) {
    QosCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.Warning,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.error,
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = stringResource(id = R.string.error_daemon_missing_title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center
            )
            Text(
                text = stringResource(id = R.string.error_daemon_missing_desc),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f),
                textAlign = TextAlign.Center
            )
            Button(onClick = onRefresh) {
                Icon(
                    imageVector = Icons.Filled.Refresh,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = stringResource(id = R.string.action_check_again))
            }
        }
    }
}

@Composable
private fun MissingConfigCard(onRefresh: () -> Unit) {
    QosCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.Error,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.error,
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = stringResource(id = R.string.error_config_missing_title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center
            )
            Text(
                text = stringResource(id = R.string.error_config_missing_desc),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f),
                textAlign = TextAlign.Center
            )
            Button(onClick = onRefresh) {
                Icon(
                    imageVector = Icons.Filled.Refresh,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(text = stringResource(id = R.string.action_check_again))
            }
        }
    }
}