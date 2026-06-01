package com.seclususs.qos.ui.components.modifiers

import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

fun Modifier.defaultScreenPadding() =
    this.fillMaxSize().statusBarsPadding().padding(horizontal = 24.dp)
        .padding(top = 2.dp, bottom = 24.dp)

@Composable
fun Modifier.iconBackground(
    size: Dp = 48.dp, color: Color = MaterialTheme.colorScheme.primary
): Modifier = this.size(size).clip(CircleShape).background(color.copy(alpha = 0.15f))

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
    return this.graphicsLayer { scaleX = scale; scaleY = scale }.clickable(
        interactionSource = interactionSource,
        indication = null,
        enabled = enabled,
        onClick = onClick
    )
}