package com.seclususs.qos.ui.navigation

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.seclususs.qos.ui.components.NavBar
import com.seclususs.qos.ui.features.advanced.AdvancedScreen
import com.seclususs.qos.ui.features.modules.ModulesScreen
import com.seclususs.qos.ui.features.services.ServicesScreen
import com.seclususs.qos.ui.features.settings.SettingsScreen

@Composable
fun QosApp() {
    val navController = rememberNavController()
    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val selectedIndex = when {
        currentDestination?.hasRoute<Services>() == true -> 0
        currentDestination?.hasRoute<Modules>() == true -> 1
        currentDestination?.hasRoute<Advanced>() == true -> 2
        currentDestination?.hasRoute<Settings>() == true -> 3
        else -> 0
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        containerColor = MaterialTheme.colorScheme.background,
        bottomBar = {
            NavBar(
                selectedIndex = selectedIndex, onItemSelected = { index ->
                    val route = when (index) {
                        0 -> Services
                        1 -> Modules
                        2 -> Advanced
                        3 -> Settings
                        else -> Services
                    }
                    val isSameRoute = when (index) {
                        0 -> currentDestination?.hasRoute<Services>() == true
                        1 -> currentDestination?.hasRoute<Modules>() == true
                        2 -> currentDestination?.hasRoute<Advanced>() == true
                        3 -> currentDestination?.hasRoute<Settings>() == true
                        else -> false
                    }
                    if (!isSameRoute) {
                        navController.navigate(route) {
                            popUpTo(navController.graph.findStartDestination().id) {
                                saveState = true
                            }
                            launchSingleTop = true
                            restoreState = true
                        }
                    }
                })
        }) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(MaterialTheme.colorScheme.background)
        ) {
            NavHost(
                navController = navController,
                startDestination = Services,
                modifier = Modifier.fillMaxSize(),
                enterTransition = {
                    fadeIn(animationSpec = tween(300)) + scaleIn(
                        initialScale = 0.80f,
                        animationSpec = tween(400, easing = FastOutSlowInEasing)
                    )
                },
                exitTransition = {
                    fadeOut(animationSpec = tween(300)) + scaleOut(
                        targetScale = 0.95f,
                        animationSpec = tween(300, easing = FastOutSlowInEasing)
                    )
                },
                popEnterTransition = {
                    fadeIn(animationSpec = tween(300)) + scaleIn(
                        initialScale = 0.80f,
                        animationSpec = tween(400, easing = FastOutSlowInEasing)
                    )
                },
                popExitTransition = {
                    fadeOut(animationSpec = tween(300)) + scaleOut(
                        targetScale = 0.95f,
                        animationSpec = tween(300, easing = FastOutSlowInEasing)
                    )
                }) {
                composable<Services> { ServicesScreen() }
                composable<Modules> { ModulesScreen() }
                composable<Advanced> { AdvancedScreen() }
                composable<Settings> { SettingsScreen() }
            }
        }
    }
}