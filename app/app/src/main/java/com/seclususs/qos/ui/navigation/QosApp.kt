package com.seclususs.qos.ui.navigation

import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.seclususs.qos.ui.components.NavBar
import com.seclususs.qos.ui.features.advanced.AdvancedScreen
import com.seclususs.qos.ui.features.modules.ModulesScreen
import com.seclususs.qos.ui.features.services.ServicesScreen
import com.seclususs.qos.ui.features.settings.SettingsScreen

@Composable
fun QosApp() {
    val navController = rememberNavController()

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        containerColor = MaterialTheme.colorScheme.background,
        bottomBar = {
            NavBar(navController = navController)
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
                enterTransition = { fadeIn(animationSpec = tween(300)) },
                exitTransition = { fadeOut(animationSpec = tween(300)) }) {
                composable<Services> {
                    ServicesScreen()
                }

                composable<Modules> {
                    ModulesScreen()
                }

                composable<Advanced> {
                    AdvancedScreen()
                }

                composable<Settings> {
                    SettingsScreen()
                }
            }
        }
    }
}