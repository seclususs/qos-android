package com.seclususs.qos.ui.components.cards

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.seclususs.qos.ui.components.modifiers.iconBackground
import com.seclususs.qos.ui.components.typography.QosSubtitleText
import com.seclususs.qos.ui.components.typography.QosTitleText

@Composable
fun ExpandableCard(
    title: String,
    icon: ImageVector,
    isExpanded: Boolean,
    onExpandClick: () -> Unit,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    trailingContent: @Composable (() -> Unit)? = null,
    content: @Composable () -> Unit
) {
    BaseCard(
        modifier = modifier.fillMaxWidth(), onClick = onExpandClick, onLongClick = {}) {
        Column(modifier = Modifier.animateContentSize(animationSpec = spring(stiffness = Spring.StiffnessLow))) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier.iconBackground(color = MaterialTheme.colorScheme.primary),
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
                    QosTitleText(title)
                    if (subtitle != null) {
                        Spacer(modifier = Modifier.height(4.dp))
                        QosSubtitleText(subtitle, maxLines = 2)
                    }
                }
                if (trailingContent != null) {
                    Spacer(modifier = Modifier.width(16.dp))
                    trailingContent()
                }
            }
            if (isExpanded) {
                HorizontalDivider(color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.05f))
                Column(
                    modifier = Modifier.padding(
                        start = 16.dp, end = 16.dp, top = 8.dp, bottom = 16.dp
                    )
                ) {
                    content()
                }
            }
        }
    }
}