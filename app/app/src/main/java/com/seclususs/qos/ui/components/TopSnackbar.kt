package com.seclususs.qos.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun TopSnackbar(
    message: String,
    isVisible: Boolean,
    modifier: Modifier = Modifier,
    icon: ImageVector = Icons.Filled.Info,
    isError: Boolean = false
) {
    val enterTransition = fadeIn(animationSpec = tween(300)) + expandHorizontally(
        expandFrom = Alignment.CenterHorizontally, animationSpec = spring(
            dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow
        )
    ) + scaleIn(
        initialScale = 0.8f, animationSpec = tween(300)
    )

    val exitTransition = fadeOut(animationSpec = tween(250)) + shrinkHorizontally(
        shrinkTowards = Alignment.CenterHorizontally, animationSpec = tween(250)
    ) + scaleOut(
        targetScale = 0.8f, animationSpec = tween(250)
    )

    AnimatedVisibility(
        visible = isVisible,
        enter = enterTransition,
        exit = exitTransition,
        modifier = modifier
            .fillMaxWidth()
            .padding(start = 24.dp, end = 24.dp, top = 16.dp, bottom = 8.dp)
    ) {
        val tintColor =
            if (isError) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary

        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .shadow(
                    elevation = 24.dp,
                    shape = RoundedCornerShape(percent = 50),
                    ambientColor = Color.Black.copy(alpha = 0.04f),
                    spotColor = Color.Black.copy(alpha = 0.04f)
                )
                .border(
                    width = 1.dp,
                    color = Color.White.copy(alpha = 0.05f),
                    shape = RoundedCornerShape(percent = 50)
                ),
            shape = RoundedCornerShape(percent = 50),
            color = MaterialTheme.colorScheme.surface
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 14.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.Start
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    tint = tintColor,
                    modifier = Modifier.size(24.dp)
                )

                Spacer(modifier = Modifier.width(12.dp))

                Text(
                    text = message,
                    color = MaterialTheme.colorScheme.onSurface,
                    style = MaterialTheme.typography.bodyLarge.copy(
                        fontWeight = FontWeight.Medium, fontSize = 14.sp
                    )
                )
            }
        }
    }
}

@Composable
fun QosTopSnackbar(
    messageResId: Int?, isError: Boolean, isVisible: Boolean, modifier: Modifier = Modifier
) {
    val snackbarMessage = messageResId?.let { stringResource(id = it) } ?: ""
    val snackbarIcon = if (isError) Icons.Filled.Error else Icons.Filled.CheckCircle
    TopSnackbar(
        message = snackbarMessage,
        isVisible = isVisible,
        isError = isError,
        icon = snackbarIcon,
        modifier = modifier
    )
}