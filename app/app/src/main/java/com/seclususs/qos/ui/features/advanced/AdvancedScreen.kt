package com.seclususs.qos.ui.features.advanced

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import com.seclususs.qos.ui.components.QosScreen
import com.seclususs.qos.ui.components.QosSubtitleText
import com.seclususs.qos.ui.components.QosTitleText
import com.seclususs.qos.ui.components.bouncyClickable
import com.seclususs.qos.ui.components.iconBackground
import com.seclususs.qos.ui.theme.scaleFadeTransition
import kotlinx.coroutines.delay

@Composable
fun AdvancedScreen(viewModel: AdvancedViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    QosScreen(
        title = stringResource(id = R.string.nav_advanced),
        snackbarMessageResId = state.snackbarMessageResId,
        snackbarIsError = state.snackbarIsError,
        snackbarVisible = state.snackbarVisible,
        isDaemonMissing = state.isDaemonMissing,
        isConfigMissing = state.isConfigMissing
    ) {
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
                FlatModifyAction(onClick = {
                    viewModel.onEvent(
                        AdvancedEvent.ShowSheet(
                            AdvancedSheetType.CPU
                        )
                    )
                })
            }
            ExpandableSwitchCard(
                title = stringResource(id = R.string.advanced_storage_title),
                icon = Icons.Filled.Storage,
                isExpanded = state.storageLimitsEnabled,
                isProcessing = state.isProcessingStorageToggle,
                onToggle = { viewModel.onEvent(AdvancedEvent.ToggleStorage(it)) }) {
                FlatModifyAction(onClick = {
                    viewModel.onEvent(
                        AdvancedEvent.ShowSheet(
                            AdvancedSheetType.STORAGE
                        )
                    )
                })
            }
            Spacer(modifier = Modifier.height(80.dp))
        }
    }

    if (state.activeSheet != null) AdvancedBottomSheet(state, viewModel)
}

@Composable
private fun FlatModifyAction(onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .bouncyClickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier.iconBackground(size = 40.dp), contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = Icons.Filled.Tune,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(20.dp)
            )
        }
        Spacer(modifier = Modifier.width(16.dp))
        QosTitleText(text = stringResource(id = R.string.advanced_action_modify))
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AdvancedBottomSheet(state: AdvancedState, viewModel: AdvancedViewModel) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val currentApplyState =
        if (state.activeSheet == AdvancedSheetType.CPU) state.cpuApplyState else state.storageApplyState
    val titleRes =
        if (state.activeSheet == AdvancedSheetType.CPU) R.string.advanced_sheet_cpu_title else R.string.advanced_sheet_storage_title

    LaunchedEffect(currentApplyState) {
        if (currentApplyState == ApplyState.SUCCESS) {
            delay(600); sheetState.hide(); viewModel.onEvent(AdvancedEvent.HideSheet)
        }
    }

    val localCpu =
        remember(state.cpuValues) { mutableStateMapOf<String, String>().apply { putAll(state.cpuValues) } }
    val localStorage =
        remember(state.storageValues) { mutableStateMapOf<String, String>().apply { putAll(state.storageValues) } }

    val cpuConfigItems = listOf(
        "latency" to R.string.advanced_label_latency to Pair("8000000", "20000000"),
        "granularity" to R.string.advanced_label_granularity to Pair("2500000", "6500000"),
        "wakeup" to R.string.advanced_label_wakeup to Pair("1500000", "6500000"),
        "migration_cost" to R.string.advanced_label_migration_cost to Pair("200000", "600000"),
        "walt_init" to R.string.advanced_label_walt_init to Pair("10", "40"),
        "uclamp_min" to R.string.advanced_label_uclamp_min to Pair("0", "384")
    )
    val storageConfigItems = listOf(
        "read_ahead" to R.string.advanced_label_read_ahead to Pair("128", "1024"),
        "nr_requests" to R.string.advanced_label_nr_requests to Pair("64", "256")
    )

    BottomSheet(
        title = stringResource(id = titleRes),
        onDismissRequest = { viewModel.onEvent(AdvancedEvent.HideSheet) },
        sheetState = sheetState
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f, fill = false)
                .windowInsetsPadding(WindowInsets.ime)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (state.activeSheet == AdvancedSheetType.CPU) {
                cpuConfigItems.forEach { (keyRes, ph) ->
                    AdvancedLimitRow(
                        stringResource(keyRes.second),
                        localCpu["${keyRes.first}_min"] ?: "",
                        localCpu["${keyRes.first}_max"] ?: "",
                        { localCpu["${keyRes.first}_min"] = it },
                        { localCpu["${keyRes.first}_max"] = it },
                        ph.first,
                        ph.second
                    )
                }
            } else {
                storageConfigItems.forEach { (keyRes, ph) ->
                    AdvancedLimitRow(
                        stringResource(keyRes.second),
                        localStorage["${keyRes.first}_min"] ?: "",
                        localStorage["${keyRes.first}_max"] ?: "",
                        { localStorage["${keyRes.first}_min"] = it },
                        { localStorage["${keyRes.first}_max"] = it },
                        ph.first,
                        ph.second
                    )
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
        }
        ApplyConfigurationButton(applyState = currentApplyState, onClick = {
            if (state.activeSheet == AdvancedSheetType.CPU) viewModel.onEvent(
                AdvancedEvent.ApplyCpuConfig(
                    localCpu.toMap()
                )
            ) else viewModel.onEvent(AdvancedEvent.ApplyStorageConfig(localStorage.toMap()))
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
            targetState = applyState, transitionSpec = scaleFadeTransition(), label = "apply"
        ) { state ->
            when (state) {
                ApplyState.LOADING -> CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    color = MaterialTheme.colorScheme.onPrimary,
                    strokeWidth = 2.dp
                )

                ApplyState.SUCCESS -> Icon(
                    imageVector = Icons.Filled.Check,
                    contentDescription = null,
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
    phMin: String,
    phMax: String
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
        QosSubtitleText(text = title, alpha = 0.8f, modifier = Modifier.padding(bottom = 6.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            NumericInputField(
                value = minVal,
                onValueChange = onMinChange,
                prefixLabel = stringResource(R.string.advanced_prefix_min),
                placeholder = phMin,
                modifier = Modifier.weight(1f)
            )
            NumericInputField(
                value = maxVal,
                onValueChange = onMaxChange,
                prefixLabel = stringResource(R.string.advanced_prefix_max),
                placeholder = phMax,
                modifier = Modifier.weight(1f)
            )
        }
    }
}