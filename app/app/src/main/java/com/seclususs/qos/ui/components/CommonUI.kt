package com.seclususs.qos.ui.components

import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

fun Modifier.defaultScreenPadding() =
    this
        .fillMaxSize()
        .statusBarsPadding()
        .padding(horizontal = 24.dp)
        .padding(top = 2.dp, bottom = 24.dp)

@Composable
fun Modifier.iconBackground(
    size: Dp = 48.dp, color: Color = MaterialTheme.colorScheme.primary
): Modifier = this
    .size(size)
    .clip(CircleShape)
    .background(color.copy(alpha = 0.15f))

@Composable
fun Modifier.bouncyClickable(
    enabled: Boolean = true, pressedScale: Float = 0.96f, onClick: () -> Unit
): Modifier {
    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (isPressed && enabled) pressedScale else 1f,
        animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
        label = "bouncyClickable"
    )
    return this
        .graphicsLayer { scaleX = scale; scaleY = scale }
        .clickable(
            interactionSource = interactionSource,
            indication = null,
            enabled = enabled,
            onClick = onClick
        )
}

@Composable
fun QosTitleText(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
        color = MaterialTheme.colorScheme.onSurface,
        modifier = modifier
    )
}

@Composable
fun QosSubtitleText(
    text: String, modifier: Modifier = Modifier, alpha: Float = 0.6f, maxLines: Int = Int.MAX_VALUE
) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge.copy(fontSize = 12.sp),
        color = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha),
        maxLines = maxLines,
        modifier = modifier
    )
}

@Composable
fun QosScreen(
    title: String,
    snackbarMessageResId: Int? = null,
    snackbarIsError: Boolean = false,
    snackbarVisible: Boolean = false,
    isDaemonMissing: Boolean = false,
    isConfigMissing: Boolean = false,
    content: @Composable () -> Unit
) {
    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier.defaultScreenPadding(),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            StateAwareContent(
                isDaemonMissing = isDaemonMissing, isConfigMissing = isConfigMissing
            ) { content() }
        }
        QosTopSnackbar(
            messageResId = snackbarMessageResId,
            isError = snackbarIsError,
            isVisible = snackbarVisible,
            modifier = Modifier.align(Alignment.TopCenter)
        )
    }
}

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
            modifier = Modifier
                .fillMaxWidth()
                .then(
                    if (onClick != null || onLongClick != null) Modifier.combinedClickable(
                        onClick = onClick ?: {}, onLongClick = onLongClick
                    )
                    else Modifier
                )
                .padding(16.dp), verticalAlignment = Alignment.CenterVertically
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