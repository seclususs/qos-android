package com.seclususs.qos.ui.features.advanced

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Memory
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.seclususs.qos.R
import com.seclususs.qos.domain.model.ConfigKeys
import com.seclususs.qos.ui.components.cards.AdvancedCard
import com.seclususs.qos.ui.components.inputs.NumericInputField
import com.seclususs.qos.ui.components.layout.BottomSheet
import com.seclususs.qos.ui.components.layout.QosScreen
import com.seclususs.qos.ui.components.typography.QosSubtitleText
import kotlinx.coroutines.launch

private data class ConfigItem(
    val minKey: String, val maxKey: String, val titleRes: Int, val minPh: String, val maxPh: String
)

private val cpuConfigItems = listOf(
    ConfigItem(
        ConfigKeys.CPU_LATENCY_MIN,
        ConfigKeys.CPU_LATENCY_MAX,
        R.string.advanced_label_latency,
        "8000000",
        "20000000"
    ), ConfigItem(
        ConfigKeys.CPU_GRANULARITY_MIN,
        ConfigKeys.CPU_GRANULARITY_MAX,
        R.string.advanced_label_granularity,
        "2500000",
        "6500000"
    ), ConfigItem(
        ConfigKeys.CPU_WAKEUP_MIN,
        ConfigKeys.CPU_WAKEUP_MAX,
        R.string.advanced_label_wakeup,
        "1500000",
        "6500000"
    ), ConfigItem(
        ConfigKeys.CPU_MIGRATION_COST_MIN,
        ConfigKeys.CPU_MIGRATION_COST_MAX,
        R.string.advanced_label_migration_cost,
        "200000",
        "600000"
    ), ConfigItem(
        ConfigKeys.CPU_WALT_INIT_MIN,
        ConfigKeys.CPU_WALT_INIT_MAX,
        R.string.advanced_label_walt_init,
        "10",
        "40"
    ), ConfigItem(
        ConfigKeys.CPU_UCLAMP_MIN_MIN,
        ConfigKeys.CPU_UCLAMP_MIN_MAX,
        R.string.advanced_label_uclamp_min,
        "0",
        "384"
    )
)

private val storageConfigItems = listOf(
    ConfigItem(
        ConfigKeys.STORAGE_READ_AHEAD_MIN,
        ConfigKeys.STORAGE_READ_AHEAD_MAX,
        R.string.advanced_label_read_ahead,
        "128",
        "1024"
    ), ConfigItem(
        ConfigKeys.STORAGE_NR_REQUESTS_MIN,
        ConfigKeys.STORAGE_NR_REQUESTS_MAX,
        R.string.advanced_label_nr_requests,
        "64",
        "256"
    )
)

@Composable
fun AdvancedScreen(viewModel: AdvancedViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val scope = rememberCoroutineScope()

    var cpuExpanded by remember { mutableStateOf(false) }
    var storageExpanded by remember { mutableStateOf(false) }

    QosScreen(
        title = stringResource(id = R.string.nav_advanced), systemStatus = state.systemStatus
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            AdvancedCard(
                title = stringResource(id = R.string.advanced_cpu_title),
                icon = Icons.Filled.Memory,
                isModuleActive = state.config.cpuEnabled,
                isExpanded = cpuExpanded,
                onExpandClick = { cpuExpanded = !cpuExpanded },
                isLimitEnabled = state.cpuLimitsEnabled,
                isProcessing = state.isProcessingCpuToggle,
                onToggle = { viewModel.onEvent(AdvancedEvent.ToggleCpu(it)) },
                description = stringResource(id = R.string.advanced_cpu_desc),
                onModifyClick = {
                    scope.launch { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.CPU)) }
                })
            AdvancedCard(
                title = stringResource(id = R.string.advanced_storage_title),
                icon = Icons.Filled.Storage,
                isModuleActive = state.config.storageEnabled,
                isExpanded = storageExpanded,
                onExpandClick = { storageExpanded = !storageExpanded },
                isLimitEnabled = state.storageLimitsEnabled,
                isProcessing = state.isProcessingStorageToggle,
                onToggle = { viewModel.onEvent(AdvancedEvent.ToggleStorage(it)) },
                description = stringResource(id = R.string.advanced_storage_desc),
                onModifyClick = {
                    scope.launch { viewModel.onEvent(AdvancedEvent.ShowSheet(AdvancedSheetType.STORAGE)) }
                })
            Spacer(modifier = Modifier.height(80.dp))
        }
    }
    if (state.activeSheet != null) AdvancedBottomSheet(state, onEvent = viewModel::onEvent)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AdvancedBottomSheet(state: AdvancedState, onEvent: (AdvancedEvent) -> Unit) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val titleRes =
        if (state.activeSheet == AdvancedSheetType.CPU) R.string.advanced_sheet_cpu_title else R.string.advanced_sheet_storage_title
    val localCpu = remember(state.activeSheet, state.cachedCpuValues) {
        mutableStateMapOf<String, String>().apply { putAll(state.cachedCpuValues) }
    }
    val localStorage = remember(state.activeSheet, state.cachedStorageValues) {
        mutableStateMapOf<String, String>().apply { putAll(state.cachedStorageValues) }
    }
    BottomSheet(
        title = stringResource(id = titleRes), onDismissRequest = {
            if (state.activeSheet == AdvancedSheetType.CPU) {
                onEvent(AdvancedEvent.SaveAndHideSheet(cpuValues = localCpu.toMap()))
            } else {
                onEvent(AdvancedEvent.SaveAndHideSheet(storageValues = localStorage.toMap()))
            }
        }, sheetState = sheetState
    ) {
        Column(
            modifier = Modifier.fillMaxWidth().weight(1f, fill = false)
                .windowInsetsPadding(WindowInsets.ime).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            if (state.activeSheet == AdvancedSheetType.CPU) {
                cpuConfigItems.forEach { item ->
                    AdvancedLimitRow(
                        stringResource(item.titleRes),
                        localCpu[item.minKey] ?: "",
                        localCpu[item.maxKey] ?: "",
                        { localCpu[item.minKey] = it },
                        { localCpu[item.maxKey] = it },
                        item.minPh,
                        item.maxPh
                    )
                }
            } else {
                storageConfigItems.forEach { item ->
                    AdvancedLimitRow(
                        stringResource(item.titleRes),
                        localStorage[item.minKey] ?: "",
                        localStorage[item.maxKey] ?: "",
                        { localStorage[item.minKey] = it },
                        { localStorage[item.maxKey] = it },
                        item.minPh,
                        item.maxPh
                    )
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
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
        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)
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