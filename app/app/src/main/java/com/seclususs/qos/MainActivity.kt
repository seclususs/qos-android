package com.seclususs.qos

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.lifecycleScope
import com.seclususs.qos.ui.navigation.QosApp
import com.seclususs.qos.ui.theme.QoSTheme
import com.topjohnwu.superuser.Shell
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        lifecycleScope.launch(Dispatchers.IO) {
            Shell.getShell()
        }
        enableEdgeToEdge()
        setContent {
            QoSTheme(darkTheme = isSystemInDarkTheme()) {
                QosApp()
            }
        }
    }
}