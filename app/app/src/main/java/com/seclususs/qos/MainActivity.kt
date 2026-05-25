package com.seclususs.qos

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.getValue
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import com.seclususs.qos.data.local.AppStore
import com.seclususs.qos.data.local.AppTheme
import com.seclususs.qos.ui.navigation.QosApp
import com.seclususs.qos.ui.theme.QoSTheme
import com.topjohnwu.superuser.Shell
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    @Inject
    lateinit var appStore: AppStore

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        lifecycleScope.launch(Dispatchers.IO) {
            Shell.getShell()
        }
        enableEdgeToEdge()
        setContent {
            val appTheme by appStore.appThemeFlow.collectAsStateWithLifecycle(initialValue = AppTheme.SYSTEM)
            val isDarkTheme = when (appTheme) {
                AppTheme.LIGHT -> false
                AppTheme.DARK -> true
                AppTheme.SYSTEM -> isSystemInDarkTheme()
            }
            QoSTheme(darkTheme = isDarkTheme) {
                QosApp()
            }
        }
    }
}