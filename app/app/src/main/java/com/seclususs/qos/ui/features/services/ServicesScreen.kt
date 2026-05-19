package com.seclususs.qos.ui.features.services

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.Crossfade
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
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
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.seclususs.qos.R
import com.seclususs.qos.ui.components.QosCard
import com.seclususs.qos.ui.theme.TechnicalTextStyle

@Composable
fun ServicesScreen(
    viewModel: ServicesViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .statusBarsPadding()
            .padding(start = 24.dp, end = 24.dp, top = 2.dp, bottom = 24.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        Text(
            text = stringResource(id = R.string.services_title),
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onBackground
        )

        Box(
            modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center
        ) {
            ReactorCore(
                status = state.status, onToggle = {
                    if (state.status == DaemonStatus.INACTIVE) {
                        viewModel.onEvent(ServicesEvent.OnStartClicked)
                    } else if (state.status == DaemonStatus.ACTIVE) {
                        viewModel.onEvent(ServicesEvent.OnStopClicked)
                    }
                })
        }

        AnimatedContent(
            targetState = state.status == DaemonStatus.MISSING, transitionSpec = {
                (fadeIn(tween(300)) + scaleIn(
                    tween(300), initialScale = 0.9f
                )) togetherWith (fadeOut(tween(200)) + scaleOut(
                    tween(200), targetScale = 0.9f
                )) using SizeTransform(
                    clip = false
                ) { _, _ ->
                    spring(
                        dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow
                    )
                }
            }, label = "missing_state_transition", modifier = Modifier.fillMaxWidth()
        ) { isMissing ->
            if (isMissing) {
                MissingDaemonCard(onRefresh = { viewModel.onEvent(ServicesEvent.RefreshMetrics) })
            } else {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(24.dp)
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
    }
}

@Composable
private fun MissingDaemonCard(onRefresh: () -> Unit) {
    QosCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.Warning,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.error,
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = stringResource(id = R.string.error_daemon_missing_title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center
            )
            Text(
                text = stringResource(id = R.string.error_daemon_missing_desc),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f),
                textAlign = TextAlign.Center
            )
            Spacer(modifier = Modifier.height(8.dp))
            ActionCard(
                title = stringResource(id = R.string.action_check_again),
                color = MaterialTheme.colorScheme.primary,
                filledIcon = Icons.Filled.Refresh,
                outlinedIcon = Icons.Outlined.Refresh,
                onClick = onRefresh
            )
        }
    }
}

@Composable
private fun ActionButtonsContainer(
    status: DaemonStatus, onRestart: () -> Unit
) {
    AnimatedContent(
        targetState = status == DaemonStatus.ACTIVE, transitionSpec = {
            (fadeIn(tween(300)) + scaleIn(tween(300), initialScale = 0.9f)) togetherWith (fadeOut(
                tween(200)
            ) + scaleOut(
                tween(200), targetScale = 0.9f
            )) using SizeTransform(clip = false) { _, _ ->
                spring(
                    dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow
                )
            }
        }, label = "action_buttons_transition", modifier = Modifier.fillMaxWidth()
    ) { isActive ->
        if (isActive) {
            ActionCard(
                title = stringResource(id = R.string.action_restart),
                color = MaterialTheme.colorScheme.primary,
                filledIcon = Icons.Filled.Refresh,
                outlinedIcon = Icons.Outlined.Refresh,
                onClick = onRestart
            )
        } else {
            Box(modifier = Modifier.fillMaxWidth())
        }
    }
}

@Composable
private fun ReactorCore(
    status: DaemonStatus, onToggle: () -> Unit
) {
    val isTransitioning =
        status == DaemonStatus.STARTING || status == DaemonStatus.STOPPING || status == DaemonStatus.RESTARTING

    val isActiveOrTransitioning = status != DaemonStatus.INACTIVE && status != DaemonStatus.MISSING
    val isMissing = status == DaemonStatus.MISSING
    val isClickable = status == DaemonStatus.ACTIVE || status == DaemonStatus.INACTIVE

    val primaryColor = MaterialTheme.colorScheme.primary
    val errorColor = MaterialTheme.colorScheme.error
    val warningColor = Color(0xFFF59E0B)
    val dimColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.15f)

    val targetColor = when (status) {
        DaemonStatus.ACTIVE -> primaryColor
        DaemonStatus.INACTIVE -> dimColor
        DaemonStatus.STARTING, DaemonStatus.RESTARTING -> warningColor
        DaemonStatus.STOPPING, DaemonStatus.MISSING -> errorColor
    }

    val animatedColor by animateColorAsState(
        targetValue = targetColor, animationSpec = tween(600), label = "reactorColor"
    )

    val infiniteTransition = rememberInfiniteTransition(label = "reactor")

    val rotation by infiniteTransition.animateFloat(
        initialValue = 0f, targetValue = 360f, animationSpec = infiniteRepeatable(
            animation = tween(
                if (isTransitioning) 1000 else if (isActiveOrTransitioning) 3000 else 10000,
                easing = LinearEasing
            ), repeatMode = RepeatMode.Restart
        ), label = "rotation"
    )

    val pulse by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = if (isTransitioning) 1.15f else if (isActiveOrTransitioning) 1.05f else 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(if (isTransitioning) 400 else 1200, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulse"
    )

    val currentRotation = if (isMissing) 0f else rotation
    val currentScale = if (isMissing) 1f else pulse

    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()

    val buttonScale by animateFloatAsState(
        targetValue = if (isPressed && isClickable) 0.92f else 1f,
        animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
        label = "reactorScale"
    )

    val statusText = when (status) {
        DaemonStatus.ACTIVE -> stringResource(id = R.string.action_stop).uppercase()
        DaemonStatus.INACTIVE -> stringResource(id = R.string.action_start).uppercase()
        DaemonStatus.STARTING -> stringResource(id = R.string.status_starting).uppercase()
        DaemonStatus.STOPPING -> stringResource(id = R.string.status_stopping).uppercase()
        DaemonStatus.RESTARTING -> stringResource(id = R.string.status_restarting).uppercase()
        DaemonStatus.MISSING -> stringResource(id = R.string.status_missing).uppercase()
    }

    Box(
        modifier = Modifier
            .size(180.dp)
            .padding(8.dp)
            .graphicsLayer {
                scaleX = buttonScale
                scaleY = buttonScale
            }
            .clip(CircleShape)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                enabled = isClickable,
                onClick = onToggle
            ), contentAlignment = Alignment.Center) {
        Box(
            modifier = Modifier
                .size(120.dp)
                .graphicsLayer {
                    scaleX = currentScale
                    scaleY = currentScale
                }
                .shadow(
                    elevation = if (isActiveOrTransitioning) 24.dp else 0.dp,
                    shape = CircleShape,
                    ambientColor = animatedColor,
                    spotColor = animatedColor
                )
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.surface))

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer { rotationZ = currentRotation }) {
            val strokeWidth = 6.dp.toPx()
            val sizeValue = size.minDimension - strokeWidth
            val offset = strokeWidth / 2f

            drawArc(
                color = animatedColor.copy(alpha = 0.3f),
                startAngle = 0f,
                sweepAngle = 360f,
                useCenter = false,
                topLeft = Offset(offset, offset),
                size = Size(sizeValue, sizeValue),
                style = Stroke(width = strokeWidth)
            )

            if (isActiveOrTransitioning) {
                drawArc(
                    color = animatedColor,
                    startAngle = 0f,
                    sweepAngle = 90f,
                    useCenter = false,
                    topLeft = Offset(offset, offset),
                    size = Size(sizeValue, sizeValue),
                    style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
                )

                drawArc(
                    color = animatedColor,
                    startAngle = 180f,
                    sweepAngle = 90f,
                    useCenter = false,
                    topLeft = Offset(offset, offset),
                    size = Size(sizeValue, sizeValue),
                    style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
                )
            }
        }

        Text(
            text = statusText,
            style = MaterialTheme.typography.labelLarge.copy(
                fontSize = 13.sp, fontWeight = FontWeight.Bold, letterSpacing = 1.5.sp
            ),
            color = if (status == DaemonStatus.INACTIVE) MaterialTheme.colorScheme.onSurface.copy(
                alpha = 0.5f
            ) else animatedColor
        )
    }
}

@Composable
private fun CustomProgressBar(
    progress: Float, color: Color, modifier: Modifier = Modifier
) {
    val animatedProgress by animateFloatAsState(
        targetValue = progress,
        animationSpec = tween(durationMillis = 800, easing = LinearOutSlowInEasing),
        label = "CustomProgressBar"
    )

    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(8.dp)
            .clip(RoundedCornerShape(percent = 50))
            .background(color.copy(alpha = 0.15f)), contentAlignment = Alignment.CenterStart
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(fraction = animatedProgress.coerceIn(0.001f, 1f))
                .height(8.dp)
                .clip(RoundedCornerShape(percent = 50))
                .background(color)
        )
    }
}

@Composable
private fun ActionCard(
    title: String,
    color: Color,
    filledIcon: ImageVector,
    outlinedIcon: ImageVector,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()

    val scale by animateFloatAsState(
        targetValue = if (isPressed) 0.96f else 1f,
        animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
        label = "buttonScale"
    )

    QosCard(
        modifier = modifier
            .fillMaxWidth()
            .height(64.dp)
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clickable(
                interactionSource = interactionSource, indication = null, onClick = onClick
            ), alpha = 0.6f) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 24.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start
        ) {
            Box(
                modifier = Modifier.size(24.dp), contentAlignment = Alignment.Center
            ) {
                Crossfade(
                    targetState = if (isPressed) filledIcon else outlinedIcon,
                    animationSpec = tween(durationMillis = 150),
                    label = "iconCrossfade"
                ) { icon ->
                    Icon(
                        imageVector = icon,
                        contentDescription = title,
                        tint = color,
                        modifier = Modifier.size(24.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.width(16.dp))

            Text(
                text = title, style = MaterialTheme.typography.labelLarge.copy(
                    fontWeight = FontWeight.SemiBold, fontSize = 15.sp
                ), color = MaterialTheme.colorScheme.onSurface
            )
        }
    }
}

@Composable
private fun TelemetryCard(
    title: String, value: String, progress: Float?, modifier: Modifier = Modifier
) {
    QosCard(
        modifier = modifier.height(96.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.Start
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelLarge.copy(fontSize = 12.sp),
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
            )
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = value,
                style = TechnicalTextStyle,
                color = MaterialTheme.colorScheme.onSurface
            )

            if (progress != null) {
                Spacer(modifier = Modifier.height(10.dp))
                CustomProgressBar(
                    progress = progress, color = MaterialTheme.colorScheme.primary
                )
            }
        }
    }
}