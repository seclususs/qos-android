package com.seclususs.qos.ui.components.cards

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.seclususs.qos.ui.components.modifiers.iconBackground
import com.seclususs.qos.ui.components.typography.QosSubtitleText
import com.seclususs.qos.ui.components.typography.QosTitleText

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun QosIconTitleCard(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    iconColor: Color? = null,
    trailingContent: @Composable (() -> Unit)? = null
) {
    val actualColor = iconColor ?: MaterialTheme.colorScheme.primary
    QosCard(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth().then(
                if (onClick != null || onLongClick != null) Modifier.combinedClickable(
                    onClick = onClick ?: {}, onLongClick = onLongClick
                )
                else Modifier
            ).padding(16.dp), verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier.iconBackground(color = actualColor),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = actualColor,
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
    }
}