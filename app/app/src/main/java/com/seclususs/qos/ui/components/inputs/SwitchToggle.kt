package com.seclususs.qos.ui.components.inputs

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
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
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import com.seclususs.qos.ui.components.modifiers.bouncyClickable

@Composable
fun SwitchToggle(
    isChecked: Boolean, isProcessing: Boolean, onToggle: () -> Unit
) {
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
        modifier = Modifier.width(switchWidth).height(switchHeight).clip(CircleShape)
            .background(trackColor)
            .bouncyClickable(enabled = !isProcessing, onClick = onToggle, onLongClick = {}),
        contentAlignment = Alignment.CenterStart
    ) {
        Box(
            modifier = Modifier.offset { IntOffset(thumbOffset.roundToPx(), 0) }.size(thumbSize)
                .shadow(elevation = 2.dp, shape = CircleShape).clip(CircleShape)
                .background(Color.White), contentAlignment = Alignment.Center
        ) {
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
                        tint = if (isChecked) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface.copy(
                            alpha = 0.4f
                        ),
                        modifier = Modifier.size(14.dp)
                    )
                }
            }
        }
    }
}