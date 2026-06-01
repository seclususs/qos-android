package com.seclususs.qos.ui.components.cards

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

@Composable
fun QosCard(
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
    alpha: Float = 1.0f,
    content: @Composable BoxScope.() -> Unit
) {
    val shape = RoundedCornerShape(24.dp)
    var currentModifier = modifier.border(
        width = 1.dp, color = Color.White.copy(alpha = 0.05f), shape = shape
    ).clip(shape)
    if (onClick != null) {
        currentModifier = currentModifier.clickable(onClick = onClick)
    }
    Surface(
        modifier = currentModifier,
        shape = shape,
        color = MaterialTheme.colorScheme.surface.copy(alpha = alpha),
        contentColor = MaterialTheme.colorScheme.onSurface
    ) {
        Box(content = content)
    }
}