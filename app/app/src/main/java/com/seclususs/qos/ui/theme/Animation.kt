package com.seclususs.qos.ui.theme

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith

fun <S> defaultSharedTransition(): AnimatedContentTransitionScope<S>.() -> ContentTransform = {
    (fadeIn(tween(300)) + scaleIn(
        tween(300), initialScale = 0.9f
    )) togetherWith (fadeOut(tween(200)) + scaleOut(
        tween(200), targetScale = 0.9f
    )) using SizeTransform(
        clip = false
    ) { _, _ ->
        spring(dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow)
    }
}