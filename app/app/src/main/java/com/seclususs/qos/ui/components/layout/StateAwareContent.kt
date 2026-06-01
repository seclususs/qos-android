package com.seclususs.qos.ui.components.layout

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.seclususs.qos.ui.animation.defaultSharedTransition
import com.seclususs.qos.ui.components.cards.MissingConfigCard
import com.seclususs.qos.ui.components.cards.MissingDaemonCard

@Composable
fun StateAwareContent(
    isDaemonMissing: Boolean,
    isConfigMissing: Boolean,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit
) {
    val targetUiState = when {
        isDaemonMissing -> 1
        isConfigMissing -> 2
        else -> 0
    }
    AnimatedContent(
        targetState = targetUiState,
        transitionSpec = defaultSharedTransition(),
        label = "state_aware_transition",
        modifier = modifier.fillMaxWidth()
    ) { uiState ->
        when (uiState) {
            1 -> MissingDaemonCard()
            2 -> MissingConfigCard()
            else -> content()
        }
    }
}