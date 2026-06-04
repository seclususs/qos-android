package com.seclususs.qos.ui.animation

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
    (fadeIn(animationSpec = tween(300)) + scaleIn(
        initialScale = 0.9f, animationSpec = tween(300)
    )) togetherWith (fadeOut(animationSpec = tween(200)) + scaleOut(
        targetScale = 0.9f, animationSpec = tween(200)
    )) using SizeTransform(clip = false) { _, _ ->
        spring(dampingRatio = Spring.DampingRatioLowBouncy, stiffness = Spring.StiffnessLow)
    }
}