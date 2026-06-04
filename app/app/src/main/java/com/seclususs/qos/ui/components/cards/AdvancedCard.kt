package com.seclususs.qos.ui.components.cards

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.seclususs.qos.R
import com.seclususs.qos.ui.components.inputs.SwitchToggle
import com.seclususs.qos.ui.components.modifiers.iconBackground
import com.seclususs.qos.ui.components.typography.QosSubtitleText
import com.seclususs.qos.ui.components.typography.QosTitleText

@Composable
fun AdvancedCard(
    title: String,
    icon: ImageVector,
    isModuleActive: Boolean,
    isExpanded: Boolean,
    onExpandClick: () -> Unit,
    isLimitEnabled: Boolean,
    isProcessing: Boolean,
    onToggle: (Boolean) -> Unit,
    description: String,
    onModifyClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    if (!isModuleActive) {
        BaseCard(
            modifier = modifier.fillMaxWidth(), onClick = null, onLongClick = null, alpha = 0.5f
        ) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier.iconBackground(color = MaterialTheme.colorScheme.onSurface),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.size(24.dp)
                    )
                }
                Spacer(modifier = Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
                    QosTitleText(title)
                    Spacer(modifier = Modifier.height(4.dp))
                    QosSubtitleText(
                        text = stringResource(id = R.string.advanced_module_disabled), maxLines = 2
                    )
                }
            }
        }
    } else {
        ExpandableCard(
            title = title,
            icon = icon,
            isExpanded = isExpanded,
            onExpandClick = onExpandClick,
            modifier = modifier,
            trailingContent = {
                SwitchToggle(
                    isChecked = isLimitEnabled,
                    isProcessing = isProcessing,
                    onToggle = { onToggle(!isLimitEnabled) })
            }) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodyLarge.copy(lineHeight = 22.sp),
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
                )
                FlatModifyAction(onClick = onModifyClick)
            }
        }
    }
}

@Composable
private fun FlatModifyAction(onClick: () -> Unit) {
    BaseCard(
        modifier = Modifier.fillMaxWidth(), onClick = onClick, onLongClick = null, alpha = 0.5f
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier.iconBackground(size = 40.dp),
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
            QosTitleText(text = stringResource(id = R.string.advanced_action_modify))
        }
    }
}