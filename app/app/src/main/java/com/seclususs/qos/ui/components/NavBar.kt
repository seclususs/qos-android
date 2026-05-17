package com.seclususs.qos.ui.components

import androidx.annotation.StringRes
import androidx.compose.animation.Crossfade
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Dns
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.outlined.Build
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.Extension
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.compose.currentBackStackEntryAsState
import com.seclususs.qos.R
import com.seclususs.qos.ui.navigation.Advanced
import com.seclususs.qos.ui.navigation.Modules
import com.seclususs.qos.ui.navigation.Services
import com.seclususs.qos.ui.navigation.Settings

private data class TopLevelRoute<out T : Any>(
    @param:StringRes val nameResId: Int,
    val route: T,
    val selectedIcon: ImageVector,
    val unselectedIcon: ImageVector
)

@Composable
fun NavBar(
    navController: NavController, modifier: Modifier = Modifier
) {
    val topLevelRoutes = listOf(
        TopLevelRoute(
            R.string.nav_services, Services, Icons.Filled.Dns, Icons.Outlined.Dns
        ),
        TopLevelRoute(
            R.string.nav_modules, Modules, Icons.Filled.Extension, Icons.Outlined.Extension
        ),
        TopLevelRoute(R.string.nav_advanced, Advanced, Icons.Filled.Build, Icons.Outlined.Build),
        TopLevelRoute(
            R.string.nav_settings, Settings, Icons.Filled.Settings, Icons.Outlined.Settings
        )
    )

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .navigationBarsPadding()
            .padding(start = 24.dp, end = 24.dp, bottom = 24.dp, top = 8.dp)
            .shadow(
                elevation = 24.dp,
                shape = RoundedCornerShape(percent = 50),
                ambientColor = Color.Black.copy(alpha = 0.04f),
                spotColor = Color.Black.copy(alpha = 0.04f)
            ),
        shape = RoundedCornerShape(percent = 50),
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.85f)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(60.dp)
                .padding(horizontal = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            topLevelRoutes.forEach { topLevelRoute ->
                val isSelected = currentDestination?.hierarchy?.any {
                    it.hasRoute(topLevelRoute.route::class)
                } == true

                CustomNavBarItem(
                    route = topLevelRoute, isSelected = isSelected, onClick = {
                        navController.navigate(topLevelRoute.route) {
                            popUpTo(Services) { saveState = true }
                            launchSingleTop = true
                            restoreState = true
                        }
                    })
            }
        }
    }
}

@Composable
private fun RowScope.CustomNavBarItem(
    route: TopLevelRoute<*>, isSelected: Boolean, onClick: () -> Unit
) {
    val interactionSource = remember { MutableInteractionSource() }

    val pillWidth by animateDpAsState(
        targetValue = if (isSelected) 88.dp else 0.dp,
        animationSpec = spring(stiffness = Spring.StiffnessMediumLow),
        label = "pillWidth"
    )

    val pillColor by animateColorAsState(
        targetValue = if (isSelected) MaterialTheme.colorScheme.primary.copy(alpha = 0.15f) else Color.Transparent,
        animationSpec = tween(durationMillis = 200),
        label = "pillColor"
    )

    val contentColor by animateColorAsState(
        targetValue = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface.copy(
            alpha = 0.4f
        ), animationSpec = tween(durationMillis = 200), label = "contentColor"
    )

    Box(
        modifier = Modifier
            .weight(1f)
            .height(60.dp)
            .clickable(
                interactionSource = interactionSource, indication = null, onClick = onClick
            ), contentAlignment = Alignment.Center
    ) {
        Box(
            modifier = Modifier
                .width(pillWidth)
                .height(48.dp)
                .background(
                    color = pillColor, shape = RoundedCornerShape(percent = 50)
                )
        )

        val labelText = stringResource(id = route.nameResId)

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Crossfade(
                targetState = if (isSelected) route.selectedIcon else route.unselectedIcon,
                animationSpec = tween(durationMillis = 200),
                label = "iconCrossfade"
            ) { currentIcon ->
                Icon(
                    imageVector = currentIcon,
                    contentDescription = labelText,
                    tint = contentColor,
                    modifier = Modifier.size(24.dp)
                )
            }

            Text(
                text = labelText,
                fontSize = 11.sp,
                lineHeight = 11.sp,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Visible,
                color = contentColor
            )
        }
    }
}