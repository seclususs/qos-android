package com.seclususs.qos.ui.navigation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.seclususs.qos.ui.features.advanced.AdvancedScreen
import com.seclususs.qos.ui.features.daemon.DaemonScreen
import com.seclususs.qos.ui.features.modules.ModulesScreen

@Composable
fun QosApp() {
    val navController = rememberNavController()
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val selectedIndex = when {
        currentDestination?.hasRoute<Daemon>() == true -> 0
        currentDestination?.hasRoute<Modules>() == true -> 1
        currentDestination?.hasRoute<Advanced>() == true -> 2
        else -> 0
    }

    val enterAnim: AnimatedContentTransitionScope<androidx.navigation.NavBackStackEntry>.() -> EnterTransition =
        {
            fadeIn(animationSpec = tween(300)) + scaleIn(
                initialScale = 0.80f, animationSpec = tween(400, easing = FastOutSlowInEasing)
            )
        }
    val exitAnim: AnimatedContentTransitionScope<androidx.navigation.NavBackStackEntry>.() -> ExitTransition =
        {
            fadeOut(animationSpec = tween(300)) + scaleOut(
                targetScale = 0.95f, animationSpec = tween(300, easing = FastOutSlowInEasing)
            )
        }

    Box(
        modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)
    ) {
        NavHost(
            navController = navController,
            startDestination = Daemon,
            modifier = Modifier.fillMaxSize(),
            enterTransition = enterAnim,
            exitTransition = exitAnim,
            popEnterTransition = enterAnim,
            popExitTransition = exitAnim
        ) {
            composable<Daemon> { DaemonScreen() }
            composable<Modules> { ModulesScreen() }
            composable<Advanced> { AdvancedScreen() }
        }
        NavBar(
            selectedIndex = selectedIndex,
            modifier = Modifier.align(Alignment.BottomCenter),
            onItemSelected = { index ->
                val route = when (index) {
                    0 -> Daemon; 1 -> Modules; else -> Advanced
                }
                if (selectedIndex != index) {
                    navController.navigate(route) {
                        popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                        launchSingleTop = true
                        restoreState = true
                    }
                }
            })
    }
}