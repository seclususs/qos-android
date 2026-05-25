package com.seclususs.qos.ui.features.advanced

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.seclususs.qos.R
import com.seclususs.qos.ui.components.BottomSheet
import com.seclususs.qos.ui.components.ExpandableSwitchCard
import com.seclususs.qos.ui.components.NumericInputField
import com.seclususs.qos.ui.components.QosCard
import com.seclususs.qos.ui.components.QosTopSnackbar
import com.seclususs.qos.ui.components.StateAwareContent
import kotlinx.coroutines.delay

@Composable
fun AdvancedScreen(
    viewModel: AdvancedViewModel = hiltViewModel()
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
                text = stringResource(id = R.string.nav_advanced),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            StateAwareContent(
                isDaemonMissing = state.isDaemonMissing, isConfigMissing = state.isConfigMissing
            ) {
                AdvancedContent(state, viewModel)
            }
        }

        if (state.activeSheet != null) {
            AdvancedBottomSheet(state = state, viewModel = viewModel)
        }

        QosTopSnackbar(
            messageResId = state.snackbarMessageResId,
            isError = state.snackbarIsError,
            isVisible = state.snackbarVisible,
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
        ExpandableSwitchCard(
            title = stringResource(id = R.string.advanced_cpu_title),
            icon = Icons.Filled.Memory,
            isExpanded = state.cpuLimitsEnabled,
            isProcessing = state.isProcessingCpuToggle,
            onToggle = { viewModel.onEvent(AdvancedEvent.ToggleCpu(it)) }) {
            ModifyButtonCard(onClick = { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.CPU)) })
        }
        ExpandableSwitchCard(
            title = stringResource(id = R.string.advanced_storage_title),
            icon = Icons.Filled.Storage,
            isExpanded = state.storageLimitsEnabled,
            isProcessing = state.isProcessingStorageToggle,
            onToggle = { viewModel.onEvent(AdvancedEvent.ToggleStorage(it)) }) {
            ModifyButtonCard(onClick = { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.STORAGE)) })
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
            Text(
                text = stringResource(id = R.string.advanced_action_modify),
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                color = MaterialTheme.colorScheme.onSurface
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AdvancedBottomSheet(state: AdvancedState, viewModel: AdvancedViewModel) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val currentApplyState =
        if (state.activeSheet == AdvancedSheetType.CPU) state.cpuApplyState else state.storageApplyState

    LaunchedEffect(currentApplyState) {
        if (currentApplyState == ApplyState.SUCCESS) {
            delay(600)
            sheetState.hide()
            viewModel.onEvent(AdvancedEvent.HideSheet)
        }
    }

    val localCpuValues = remember(state.cpuValues) {
        mutableStateMapOf<String, String>().apply { putAll(state.cpuValues) }
    }
    val localStorageValues = remember(state.storageValues) {
        mutableStateMapOf<String, String>().apply { putAll(state.storageValues) }
    }

    val cpuConfigItems = listOf(
        Triple("latency", R.string.advanced_label_latency, Pair("8000000", "20000000")),
        Triple("granularity", R.string.advanced_label_granularity, Pair("2500000", "6500000")),
        Triple("wakeup", R.string.advanced_label_wakeup, Pair("1500000", "6500000")),
        Triple("migration_cost", R.string.advanced_label_migration_cost, Pair("200000", "600000")),
        Triple("walt_init", R.string.advanced_label_walt_init, Pair("10", "40")),
        Triple("uclamp_min", R.string.advanced_label_uclamp_min, Pair("0", "384"))
    )

    val storageConfigItems = listOf(
        Triple("read_ahead", R.string.advanced_label_read_ahead, Pair("128", "1024")),
        Triple("nr_requests", R.string.advanced_label_nr_requests, Pair("64", "256"))
    )

    BottomSheet(
        onDismissRequest = { viewModel.onEvent(AdvancedEvent.HideSheet) }, sheetState = sheetState
    ) {
        val title =
            if (state.activeSheet == AdvancedSheetType.CPU) stringResource(id = R.string.advanced_sheet_cpu_title) else stringResource(
                id = R.string.advanced_sheet_storage_title
            )
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
                cpuConfigItems.forEach { (key, titleRes, placeholders) ->
                    AdvancedLimitRow(
                        title = stringResource(id = titleRes),
                        minVal = localCpuValues["${key}_min"] ?: "",
                        maxVal = localCpuValues["${key}_max"] ?: "",
                        onMinChange = { localCpuValues["${key}_min"] = it },
                        onMaxChange = { localCpuValues["${key}_max"] = it },
                        placeholderMin = placeholders.first,
                        placeholderMax = placeholders.second
                    )
                }
            } else if (state.activeSheet == AdvancedSheetType.STORAGE) {
                storageConfigItems.forEach { (key, titleRes, placeholders) ->
                    AdvancedLimitRow(
                        title = stringResource(id = titleRes),
                        minVal = localStorageValues["${key}_min"] ?: "",
                        maxVal = localStorageValues["${key}_max"] ?: "",
                        onMinChange = { localStorageValues["${key}_min"] = it },
                        onMaxChange = { localStorageValues["${key}_max"] = it },
                        placeholderMin = placeholders.first,
                        placeholderMax = placeholders.second
                    )
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
        }

        ApplyConfigurationButton(
            applyState = currentApplyState, onClick = {
                if (state.activeSheet == AdvancedSheetType.CPU) viewModel.onEvent(
                    AdvancedEvent.ApplyCpuConfig(
                        localCpuValues.toMap()
                    )
                )
                else viewModel.onEvent(AdvancedEvent.ApplyStorageConfig(localStorageValues.toMap()))
            })
    }
}

@Composable
private fun ApplyConfigurationButton(applyState: ApplyState, onClick: () -> Unit) {
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
                (fadeIn(tween(200)) + scaleIn(tween(200))) togetherWith (fadeOut(tween(200)) + scaleOut(
                    tween(200)
                ))
            }, label = "apply_button_animation"
        ) { state ->
            when (state) {
                ApplyState.LOADING -> CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = MaterialTheme.colorScheme.onPrimary,
                    strokeWidth = 2.dp
                )

                ApplyState.SUCCESS -> Icon(
                    imageVector = Icons.Filled.Check,
                    contentDescription = "Success",
                    modifier = Modifier.size(20.dp)
                )

                ApplyState.IDLE -> Text(
                    text = stringResource(id = R.string.action_apply_config),
                    style = MaterialTheme.typography.labelLarge.copy(
                        fontWeight = FontWeight.SemiBold, fontSize = 16.sp
                    )
                )
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