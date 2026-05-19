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
import com.seclususs.qos.R

enum class TopLevelRoute(
    @param:StringRes val nameResId: Int,
    val selectedIcon: ImageVector,
    val unselectedIcon: ImageVector
) {
    SERVICES(
        R.string.nav_services, Icons.Filled.Dns, Icons.Outlined.Dns
    ),
    MODULES(
        R.string.nav_modules, Icons.Filled.Extension, Icons.Outlined.Extension
    ),
    ADVANCED(
        R.string.nav_advanced, Icons.Filled.Build, Icons.Outlined.Build
    ),
    SETTINGS(R.string.nav_settings, Icons.Filled.Settings, Icons.Outlined.Settings)
}

@Composable
fun NavBar(
    selectedIndex: Int, onItemSelected: (Int) -> Unit, modifier: Modifier = Modifier
) {
    val topLevelRoutes = TopLevelRoute.entries

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
            topLevelRoutes.forEachIndexed { index, route ->
                CustomNavBarItem(
                    route = route,
                    isSelected = selectedIndex == index,
                    onClick = { onItemSelected(index) })
            }
        }
    }
}

@Composable
private fun RowScope.CustomNavBarItem(
    route: TopLevelRoute, isSelected: Boolean, onClick: () -> Unit
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