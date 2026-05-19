package com.seclususs.qos.ui.features.advanced

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
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
import androidx.compose.ui.text.input.KeyboardType
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
fun AdvancedScreen(
    viewModel: AdvancedViewModel = hiltViewModel()
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
                text = stringResource(id = R.string.nav_advanced),
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
                }, label = "advanced_ui_state", modifier = Modifier.fillMaxWidth()
            ) { uiState ->
                when (uiState) {
                    1 -> MissingDaemonCard(onRefresh = { viewModel.onEvent(AdvancedEvent.RefreshStatus) })
                    2 -> MissingConfigCard(onRefresh = { viewModel.onEvent(AdvancedEvent.RefreshStatus) })
                    else -> AdvancedContent(state, viewModel)
                }
            }
        }

        if (state.activeSheet != null) {
            AdvancedBottomSheet(state = state, viewModel = viewModel)
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
    }
}

@Composable
private fun AdvancedContent(state: AdvancedState, viewModel: AdvancedViewModel) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        ExpandableLimitsCard(
            title = stringResource(id = R.string.advanced_cpu_title),
            icon = Icons.Filled.Memory,
            isExpanded = state.cpuLimitsEnabled,
            isProcessing = state.isProcessingCpuToggle,
            onToggle = { viewModel.onEvent(AdvancedEvent.ToggleCpu(it)) }) {
            ModifyButtonCard(
                onClick = { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.CPU)) })
        }

        ExpandableLimitsCard(
            title = stringResource(id = R.string.advanced_storage_title),
            icon = Icons.Filled.Storage,
            isExpanded = state.storageLimitsEnabled,
            isProcessing = state.isProcessingStorageToggle,
            onToggle = { viewModel.onEvent(AdvancedEvent.ToggleStorage(it)) }) {
            ModifyButtonCard(
                onClick = { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.STORAGE)) })
        }

        Spacer(modifier = Modifier.height(80.dp))
    }
}

@Composable
private fun ModifyButtonCard(onClick: () -> Unit) {
    QosCard(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        alpha = 0.2f,
        onClick = onClick
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.1f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Filled.Tune,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(20.dp)
                )
            }

            Spacer(modifier = Modifier.width(16.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(id = R.string.advanced_action_modify),
                    style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.onSurface
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AdvancedBottomSheet(state: AdvancedState, viewModel: AdvancedViewModel) {
    BottomSheet(onDismissRequest = { viewModel.onEvent(AdvancedEvent.HideSheet) }) {
        val title = if (state.activeSheet == AdvancedSheetType.CPU) {
            stringResource(id = R.string.advanced_sheet_cpu_title)
        } else {
            stringResource(id = R.string.advanced_sheet_storage_title)
        }

        Text(
            text = title,
            style = MaterialTheme.typography.titleLarge.copy(fontSize = 20.sp),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f, fill = false)
                .windowInsetsPadding(WindowInsets.ime)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (state.activeSheet == AdvancedSheetType.CPU) {
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_latency),
                    minVal = state.cpuValues["latency_min"] ?: "",
                    maxVal = state.cpuValues["latency_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "latency_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "latency_max", it
                            )
                        )
                    },
                    placeholderMin = "8000000",
                    placeholderMax = "20000000"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_granularity),
                    minVal = state.cpuValues["granularity_min"] ?: "",
                    maxVal = state.cpuValues["granularity_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "granularity_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "granularity_max", it
                            )
                        )
                    },
                    placeholderMin = "2500000",
                    placeholderMax = "6500000"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_wakeup),
                    minVal = state.cpuValues["wakeup_min"] ?: "",
                    maxVal = state.cpuValues["wakeup_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "wakeup_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "wakeup_max", it
                            )
                        )
                    },
                    placeholderMin = "1500000",
                    placeholderMax = "6500000"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_migration_cost),
                    minVal = state.cpuValues["migration_cost_min"] ?: "",
                    maxVal = state.cpuValues["migration_cost_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "migration_cost_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "migration_cost_max", it
                            )
                        )
                    },
                    placeholderMin = "200000",
                    placeholderMax = "600000"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_walt_init),
                    minVal = state.cpuValues["walt_init_min"] ?: "",
                    maxVal = state.cpuValues["walt_init_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "walt_init_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "walt_init_max", it
                            )
                        )
                    },
                    placeholderMin = "10",
                    placeholderMax = "40"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_uclamp_min),
                    minVal = state.cpuValues["uclamp_min_min"] ?: "",
                    maxVal = state.cpuValues["uclamp_min_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "uclamp_min_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateCpuField(
                                "uclamp_min_max", it
                            )
                        )
                    },
                    placeholderMin = "0",
                    placeholderMax = "384"
                )
            } else if (state.activeSheet == AdvancedSheetType.STORAGE) {
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_read_ahead),
                    minVal = state.storageValues["read_ahead_min"] ?: "",
                    maxVal = state.storageValues["read_ahead_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateStorageField(
                                "read_ahead_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateStorageField(
                                "read_ahead_max", it
                            )
                        )
                    },
                    placeholderMin = "128",
                    placeholderMax = "1024"
                )
                AdvancedLimitRow(
                    title = stringResource(id = R.string.advanced_label_nr_requests),
                    minVal = state.storageValues["nr_requests_min"] ?: "",
                    maxVal = state.storageValues["nr_requests_max"] ?: "",
                    onMinChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateStorageField(
                                "nr_requests_min", it
                            )
                        )
                    },
                    onMaxChange = {
                        viewModel.onEvent(
                            AdvancedEvent.UpdateStorageField(
                                "nr_requests_max", it
                            )
                        )
                    },
                    placeholderMin = "64",
                    placeholderMax = "256"
                )
            }
            Spacer(modifier = Modifier.height(16.dp))
        }

        ApplyConfigurationButton(
            applyState = if (state.activeSheet == AdvancedSheetType.CPU) state.cpuApplyState else state.storageApplyState,
            onClick = {
                if (state.activeSheet == AdvancedSheetType.CPU) viewModel.onEvent(AdvancedEvent.ApplyCpuConfig)
                else viewModel.onEvent(AdvancedEvent.ApplyStorageConfig)
            })
    }
}

@Composable
private fun ExpandableLimitsCard(
    title: String,
    icon: ImageVector,
    isExpanded: Boolean,
    isProcessing: Boolean,
    onToggle: (Boolean) -> Unit,
    content: @Composable () -> Unit
) {
    QosCard(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.animateContentSize(animationSpec = spring(stiffness = Spring.StiffnessLow))) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = { if (!isProcessing) onToggle(!isExpanded) })
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
                Text(
                    text = title,
                    style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f)
                )
                Spacer(modifier = Modifier.width(16.dp))
                AnimatedQosSwitch(isChecked = isExpanded, isProcessing = isProcessing)
            }

            if (isExpanded) {
                HorizontalDivider(color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.05f))
                Column(modifier = Modifier.padding(bottom = 8.dp)) {
                    content()
                }
            }
        }
    }
}

@Composable
private fun ApplyConfigurationButton(
    applyState: ApplyState, onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        shape = RoundedCornerShape(12.dp),
        colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary),
        contentPadding = PaddingValues(horizontal = 20.dp, vertical = 14.dp),
        enabled = applyState != ApplyState.LOADING,
        modifier = Modifier
            .fillMaxWidth()
            .animateContentSize()
    ) {
        AnimatedContent(
            targetState = applyState, transitionSpec = {
                (fadeIn(tween(200)) + scaleIn(tween(200))) togetherWith (fadeOut(
                    tween(200)
                ) + scaleOut(tween(200)))
            }, label = "apply_button_animation"
        ) { state ->
            when (state) {
                ApplyState.LOADING -> {
                    CircularProgressIndicator(
                        modifier = Modifier.size(20.dp),
                        color = MaterialTheme.colorScheme.onPrimary,
                        strokeWidth = 2.dp
                    )
                }

                ApplyState.SUCCESS -> {
                    Icon(
                        imageVector = Icons.Filled.Check,
                        contentDescription = "Success",
                        modifier = Modifier.size(20.dp)
                    )
                }

                ApplyState.IDLE -> {
                    Text(
                        text = stringResource(id = R.string.action_apply_config),
                        style = MaterialTheme.typography.labelLarge.copy(
                            fontWeight = FontWeight.SemiBold, fontSize = 16.sp
                        )
                    )
                }
            }
        }
    }
}

@Composable
private fun AdvancedLimitRow(
    title: String,
    minVal: String,
    maxVal: String,
    onMinChange: (String) -> Unit,
    onMaxChange: (String) -> Unit,
    placeholderMin: String,
    placeholderMax: String
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f),
            modifier = Modifier.padding(bottom = 6.dp)
        )
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            NumericInputField(
                value = minVal,
                onValueChange = onMinChange,
                prefixLabel = stringResource(id = R.string.advanced_prefix_min),
                placeholder = placeholderMin,
                modifier = Modifier.weight(1f)
            )
            NumericInputField(
                value = maxVal,
                onValueChange = onMaxChange,
                prefixLabel = stringResource(id = R.string.advanced_prefix_max),
                placeholder = placeholderMax,
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun NumericInputField(
    value: String,
    onValueChange: (String) -> Unit,
    prefixLabel: String,
    placeholder: String,
    modifier: Modifier = Modifier
) {
    OutlinedTextField(
        value = value,
        onValueChange = { newVal -> onValueChange(newVal.filter { it.isDigit() }) },
        prefix = {
            Text(
                text = prefixLabel,
                color = MaterialTheme.colorScheme.primary,
                style = MaterialTheme.typography.labelLarge
            )
        },
        placeholder = {
            Text(
                text = placeholder, color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
            )
        },
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
        shape = RoundedCornerShape(12.dp),
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = MaterialTheme.colorScheme.primary,
            unfocusedBorderColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.1f),
            focusedContainerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.5f),
            unfocusedContainerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.2f)
        ),
        modifier = modifier.height(52.dp)
    )
}

@Composable
private fun AnimatedQosSwitch(isChecked: Boolean, isProcessing: Boolean) {
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