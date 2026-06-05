package com.seclususs.qos.ui.components.layout

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.seclususs.qos.domain.model.SystemStatus
import com.seclususs.qos.ui.animation.defaultSharedTransition
import com.seclususs.qos.ui.components.cards.MissingConfigCard
import com.seclususs.qos.ui.components.cards.MissingDaemonCard

@Composable
fun StateAwareContent(
    systemStatus: SystemStatus, modifier: Modifier = Modifier, content: @Composable () -> Unit
) {
    AnimatedContent(
        targetState = systemStatus,
        transitionSpec = defaultSharedTransition(),
        label = "state_aware_transition",
        modifier = modifier.fillMaxWidth()
    ) { state ->
        when (state) {
            SystemStatus.DAEMON_MISSING -> MissingDaemonCard()
            SystemStatus.CONFIG_MISSING -> MissingConfigCard()
            SystemStatus.OK -> content()
        }
    }
}