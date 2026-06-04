package com.seclususs.qos.ui.navigation

import androidx.annotation.Keep
import androidx.annotation.StringRes
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.filled.Layers
import androidx.compose.material.icons.outlined.Build
import androidx.compose.material.icons.outlined.Extension
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.ui.graphics.vector.ImageVector
import com.seclususs.qos.R
import kotlinx.serialization.Serializable

@Keep
@Serializable
object Daemon

@Keep
@Serializable
object Modules

@Keep
@Serializable
object Advanced

enum class TopLevelRoute(
    @param:StringRes val nameResId: Int,
    val selectedIcon: ImageVector,
    val unselectedIcon: ImageVector
) {
    DAEMON(
        R.string.nav_daemon, Icons.Filled.Layers, Icons.Outlined.Layers
    ),
    MODULES(
        R.string.nav_modules, Icons.Filled.Extension, Icons.Outlined.Extension
    ),
    ADVANCED(
        R.string.nav_advanced, Icons.Filled.Build, Icons.Outlined.Build
    )
}