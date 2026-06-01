package com.seclususs.qos.ui.features.services

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.seclususs.qos.R
import com.seclususs.qos.ui.components.cards.ActionCard
import com.seclususs.qos.ui.components.cards.TelemetryCard
import com.seclususs.qos.ui.components.modifiers.bouncyClickable
import com.seclususs.qos.ui.components.modifiers.defaultScreenPadding

@Composable
fun ServicesScreen(viewModel: ServicesViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val uriHandler = LocalUriHandler.current

    Column(
        modifier = Modifier.defaultScreenPadding(),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = stringResource(id = R.string.services_title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            Icon(
                painter = painterResource(id = R.drawable.ic_github),
                contentDescription = "Open GitHub",
                tint = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.size(26.dp)
                    .bouncyClickable { uriHandler.openUri("https://github.com/seclususs") })
        }

        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            ReactorCore(status = state.status, onToggle = {
                if (state.status == DaemonStatus.INACTIVE) viewModel.onEvent(ServicesEvent.OnStartClicked)
                else if (state.status == DaemonStatus.ACTIVE) viewModel.onEvent(ServicesEvent.OnStopClicked)
            })
        }

        Column(
            modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            ActionButtonsContainer(
                status = state.status,
                onRestart = { viewModel.onEvent(ServicesEvent.OnRestartClicked) })

            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    TelemetryCard(
                        title = stringResource(id = R.string.metric_cpu),
                        value = state.cpuUsage,
                        progress = state.cpuProgress,
                        modifier = Modifier.weight(1f)
                    )
                    TelemetryCard(
                        title = stringResource(id = R.string.metric_ram),
                        value = state.ramUsage,
                        progress = state.ramProgress,
                        modifier = Modifier.weight(1f)
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    TelemetryCard(
                        title = stringResource(id = R.string.metric_uptime),
                        value = state.uptime,
                        progress = null,
                        modifier = Modifier.weight(1f)
                    )
                    TelemetryCard(
                        title = stringResource(id = R.string.metric_pid),
                        value = state.pid,
                        progress = null,
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }
    }
}

@Composable
private fun ActionButtonsContainer(status: DaemonStatus, onRestart: () -> Unit) {
    AnimatedContent(
        targetState = status == DaemonStatus.ACTIVE, transitionSpec = {
            (fadeIn(animationSpec = tween(300)) + scaleIn(
                initialScale = 0.9f, animationSpec = tween(300)
            )) togetherWith (fadeOut(animationSpec = tween(200)) + scaleOut(
                targetScale = 0.9f, animationSpec = tween(200)
            )) using SizeTransform(clip = false) { _, _ ->
                spring(
                    dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow
                )
            }
        }, label = "action_btn", modifier = Modifier.fillMaxWidth()
    ) { isActive ->
        if (isActive) ActionCard(
            title = stringResource(id = R.string.action_restart),
            color = MaterialTheme.colorScheme.primary,
            filledIcon = Icons.Filled.Refresh,
            outlinedIcon = Icons.Outlined.Refresh,
            onClick = onRestart
        )
        else Box(
            modifier = Modifier.fillMaxWidth().height(0.dp)
        )
    }
}

@Composable
private fun ReactorCore(status: DaemonStatus, onToggle: () -> Unit) {
    val isTransitioning =
        status in listOf(DaemonStatus.STARTING, DaemonStatus.STOPPING, DaemonStatus.RESTARTING)
    val isActiveOrTransitioning = status != DaemonStatus.INACTIVE && status != DaemonStatus.MISSING
    val isMissing = status == DaemonStatus.MISSING
    val isClickable = status == DaemonStatus.ACTIVE || status == DaemonStatus.INACTIVE

    val animatedColor by animateColorAsState(
        targetValue = when (status) {
            DaemonStatus.ACTIVE -> MaterialTheme.colorScheme.primary
            DaemonStatus.INACTIVE -> MaterialTheme.colorScheme.onSurface.copy(alpha = 0.15f)
            DaemonStatus.STARTING, DaemonStatus.RESTARTING -> MaterialTheme.colorScheme.tertiary
            DaemonStatus.STOPPING, DaemonStatus.MISSING -> MaterialTheme.colorScheme.error
        }, animationSpec = tween(600), label = "color"
    )
    val infiniteTransition = rememberInfiniteTransition(label = "reactor")
    val rotation by infiniteTransition.animateFloat(
        initialValue = 0f, targetValue = 360f, animationSpec = infiniteRepeatable(
            animation = tween(
                if (isTransitioning) 1000 else if (isActiveOrTransitioning) 3000 else 10000,
                easing = LinearEasing
            )
        ), label = "rot"
    )
    val pulse by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (isTransitioning) 1.15f else if (isActiveOrTransitioning) 1.05f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(
                if (isTransitioning) 400 else 1200, easing = FastOutSlowInEasing
            ), repeatMode = RepeatMode.Reverse
        ),
        label = "pulse"
    )

    val currentRotation = if (isMissing) 0f else rotation
    val currentScale = if (isMissing) 1f else pulse

    val statusText = stringResource(
        id = when (status) {
            DaemonStatus.ACTIVE -> R.string.action_stop
            DaemonStatus.INACTIVE -> R.string.action_start
            DaemonStatus.STARTING -> R.string.status_starting
            DaemonStatus.STOPPING -> R.string.status_stopping
            DaemonStatus.RESTARTING -> R.string.status_restarting
            DaemonStatus.MISSING -> R.string.status_missing
        }
    ).uppercase()

    val density = LocalDensity.current
    val strokeWidthPx = remember(density) { with(density) { 6.dp.toPx() } }

    Box(
        modifier = Modifier.size(180.dp).padding(8.dp).clip(CircleShape)
            .bouncyClickable(enabled = isClickable, onClick = onToggle),
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier.size(120.dp)
                .graphicsLayer { scaleX = currentScale; scaleY = currentScale }.shadow(
                    elevation = if (isActiveOrTransitioning) 24.dp else 0.dp,
                    shape = CircleShape,
                    ambientColor = animatedColor,
                    spotColor = animatedColor
                ).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
        )

        Canvas(
            modifier = Modifier.fillMaxSize().graphicsLayer { rotationZ = currentRotation }) {
            val sizeValue = size.minDimension - strokeWidthPx
            val offset = strokeWidthPx / 2f
            drawArc(
                color = animatedColor.copy(alpha = 0.3f),
                startAngle = 0f,
                sweepAngle = 360f,
                useCenter = false,
                topLeft = Offset(offset, offset),
                size = Size(sizeValue, sizeValue),
                style = Stroke(width = strokeWidthPx)
            )
            if (isActiveOrTransitioning) {
                drawArc(
                    color = animatedColor,
                    startAngle = 0f,
                    sweepAngle = 90f,
                    useCenter = false,
                    topLeft = Offset(offset, offset),
                    size = Size(sizeValue, sizeValue),
                    style = Stroke(width = strokeWidthPx, cap = StrokeCap.Round)
                )
                drawArc(
                    color = animatedColor,
                    startAngle = 180f,
                    sweepAngle = 90f,
                    useCenter = false,
                    topLeft = Offset(offset, offset),
                    size = Size(sizeValue, sizeValue),
                    style = Stroke(width = strokeWidthPx, cap = StrokeCap.Round)
                )
            }
        }

        Text(
            text = statusText,
            color = if (status == DaemonStatus.INACTIVE) MaterialTheme.colorScheme.onSurface.copy(
                alpha = 0.5f
            ) else animatedColor,
            style = MaterialTheme.typography.labelLarge.copy(
                fontSize = 13.sp, fontWeight = FontWeight.Bold, letterSpacing = 1.5.sp
            )
        )
    }
}