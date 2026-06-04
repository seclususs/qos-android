package com.seclususs.qos.ui.features.daemon

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
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
import androidx.compose.material.icons.filled.PowerSettingsNew
import androidx.compose.material.icons.outlined.PowerSettingsNew
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
import com.seclususs.qos.ui.animation.defaultSharedTransition
import com.seclususs.qos.ui.components.cards.ActionCard
import com.seclususs.qos.ui.components.cards.InfoCard
import com.seclususs.qos.ui.components.modifiers.bouncyClickable
import com.seclususs.qos.ui.components.modifiers.defaultScreenPadding

@Composable
fun DaemonScreen(viewModel: DaemonViewModel = hiltViewModel()) {
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
                text = stringResource(id = R.string.daemon_title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
            Icon(
                painter = painterResource(id = R.drawable.ic_github),
                contentDescription = "Open GitHub",
                tint = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.size(26.dp).bouncyClickable(
                    onClick = { uriHandler.openUri("https://github.com/seclususs") })
            )
        }

        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            ReactorCore(status = state.status, onToggle = {
                if (state.status == DaemonStatus.ACTIVE) viewModel.onEvent(DaemonEvent.OnStopClicked)
            })
        }

        Column(
            modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            ActionButtonsContainer(
                showReboot = state.needsReboot || state.status == DaemonStatus.INACTIVE,
                onReboot = { viewModel.onEvent(DaemonEvent.OnRebootClicked) })
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                InfoCard(
                    title = stringResource(id = R.string.metric_uptime), value = state.uptime
                )
                InfoCard(
                    title = stringResource(id = R.string.metric_pid), value = state.pid
                )
            }
        }
    }
}

@Composable
private fun ActionButtonsContainer(showReboot: Boolean, onReboot: () -> Unit) {
    AnimatedContent(
        targetState = showReboot,
        transitionSpec = defaultSharedTransition(),
        label = "action_btn",
        modifier = Modifier.fillMaxWidth()
    ) { isNeeded ->
        if (isNeeded) ActionCard(
            title = stringResource(id = R.string.action_reboot),
            color = MaterialTheme.colorScheme.tertiary,
            filledIcon = Icons.Filled.PowerSettingsNew,
            outlinedIcon = Icons.Outlined.PowerSettingsNew,
            onClick = onReboot
        )
        else Box(
            modifier = Modifier.fillMaxWidth().height(0.dp)
        )
    }
}

@Composable
private fun ReactorCore(status: DaemonStatus, onToggle: () -> Unit) {
    val isTransitioning = status == DaemonStatus.STOPPING || status == DaemonStatus.SEARCHING
    val isMissing = status == DaemonStatus.MISSING || status == DaemonStatus.INACTIVE
    val isClickable = status == DaemonStatus.ACTIVE
    val showArcsAndShadow = status == DaemonStatus.ACTIVE || isTransitioning
    val animatedColor by animateColorAsState(
        targetValue = when (status) {
            DaemonStatus.ACTIVE -> MaterialTheme.colorScheme.primary
            DaemonStatus.STOPPING -> MaterialTheme.colorScheme.tertiary
            DaemonStatus.SEARCHING -> MaterialTheme.colorScheme.tertiary
            DaemonStatus.INACTIVE, DaemonStatus.MISSING -> MaterialTheme.colorScheme.error
        }, animationSpec = tween(600), label = "color"
    )
    val infiniteTransition = rememberInfiniteTransition(label = "reactor")
    val rotation by infiniteTransition.animateFloat(
        initialValue = 0f, targetValue = 360f, animationSpec = infiniteRepeatable(
            animation = tween(
                if (isTransitioning) 1000 else if (status == DaemonStatus.ACTIVE) 3000 else 10000,
                easing = LinearEasing
            )
        ), label = "rot"
    )
    val pulse by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (isTransitioning) 1.15f else if (status == DaemonStatus.ACTIVE) 1.05f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(if (isTransitioning) 400 else 1200, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulse"
    )

    val currentRotation = if (isMissing) 0f else rotation
    val currentScale = if (isMissing) 1f else pulse

    val statusText = stringResource(
        id = when (status) {
            DaemonStatus.ACTIVE -> R.string.action_stop
            DaemonStatus.INACTIVE -> R.string.status_stopped
            DaemonStatus.STOPPING -> R.string.status_stopping
            DaemonStatus.MISSING -> R.string.status_missing
            DaemonStatus.SEARCHING -> R.string.status_searching
        }
    ).uppercase()

    val density = LocalDensity.current
    val strokeWidthPx = remember(density) { with(density) { 6.dp.toPx() } }

    Box(
        modifier = Modifier.size(220.dp).padding(8.dp).clip(CircleShape)
            .bouncyClickable(enabled = isClickable, onClick = onToggle),
        contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier.size(150.dp)
                .graphicsLayer { scaleX = currentScale; scaleY = currentScale }.shadow(
                    elevation = if (showArcsAndShadow) 24.dp else 0.dp,
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
            if (showArcsAndShadow) {
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
            color = animatedColor,
            style = MaterialTheme.typography.labelLarge.copy(
                fontSize = 14.sp, fontWeight = FontWeight.Bold, letterSpacing = 1.5.sp
            )
        )
    }
}