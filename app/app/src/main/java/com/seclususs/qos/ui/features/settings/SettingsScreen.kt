package com.seclususs.qos.ui.features.settings

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.seclususs.qos.R
import com.seclususs.qos.data.local.AppTheme
import com.seclususs.qos.ui.components.BottomSheet
import com.seclususs.qos.ui.components.QosCard
import com.seclususs.qos.ui.components.TopSnackbar

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: SettingsViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsState()

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .padding(start = 24.dp, end = 24.dp, top = 2.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Text(
                text = stringResource(id = R.string.nav_settings),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            QosCard(
                modifier = Modifier.fillMaxWidth(),
                onClick = { viewModel.onEvent(SettingsEvent.OnThemeCardClicked) }) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .clip(RoundedCornerShape(percent = 50))
                            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Palette,
                            contentDescription = stringResource(id = R.string.settings_theme_title),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }

                    Spacer(modifier = Modifier.width(16.dp))

                    Column(
                        modifier = Modifier.weight(1f), verticalArrangement = Arrangement.Center
                    ) {
                        Text(
                            text = stringResource(id = R.string.settings_theme_title),
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                            color = MaterialTheme.colorScheme.onSurface
                        )
                    }
                }
            }
        }

        val snackbarMessage = state.snackbarMessageResId?.let { stringResource(id = it) } ?: ""
        val snackbarIcon =
            if (state.snackbarIsError) Icons.Filled.Error else Icons.Filled.CheckCircle

        TopSnackbar(
            message = snackbarMessage,
            isVisible = state.snackbarVisible,
            isError = state.snackbarIsError,
            icon = snackbarIcon,
            modifier = Modifier.align(Alignment.TopCenter)
        )

        if (state.showThemeSheet) {
            BottomSheet(
                onDismissRequest = { viewModel.onEvent(SettingsEvent.OnDismissThemeSheet) }) {
                Text(
                    text = stringResource(id = R.string.settings_select_theme),
                    style = MaterialTheme.typography.titleLarge.copy(fontSize = 20.sp),
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(bottom = 16.dp)
                )
                ThemeSelectionItem(
                    label = stringResource(id = R.string.theme_system),
                    isSelected = state.appTheme == AppTheme.SYSTEM,
                    isProcessing = state.processingTheme == AppTheme.SYSTEM,
                    onClick = { viewModel.onEvent(SettingsEvent.OnThemeSelected(AppTheme.SYSTEM)) })
                ThemeSelectionItem(
                    label = stringResource(id = R.string.theme_light),
                    isSelected = state.appTheme == AppTheme.LIGHT,
                    isProcessing = state.processingTheme == AppTheme.LIGHT,
                    onClick = { viewModel.onEvent(SettingsEvent.OnThemeSelected(AppTheme.LIGHT)) })
                ThemeSelectionItem(
                    label = stringResource(id = R.string.theme_dark),
                    isSelected = state.appTheme == AppTheme.DARK,
                    isProcessing = state.processingTheme == AppTheme.DARK,
                    onClick = { viewModel.onEvent(SettingsEvent.OnThemeSelected(AppTheme.DARK)) })
            }
        }
    }
}

@Composable
private fun ThemeSelectionItem(
    label: String, isSelected: Boolean, isProcessing: Boolean, onClick: () -> Unit
) {
    val bgColor =
        if (isSelected) MaterialTheme.colorScheme.primary.copy(alpha = 0.15f) else Color.Transparent
    val textColor =
        if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
            .clip(RoundedCornerShape(percent = 50))
            .background(bgColor)
            .clickable(onClick = onClick, enabled = !isProcessing)
            .padding(horizontal = 20.dp, vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium),
            color = textColor
        )

        Box(modifier = Modifier.size(24.dp), contentAlignment = Alignment.Center) {
            AnimatedContent(
                targetState = isProcessing, transitionSpec = {
                    (fadeIn(tween(200)) + scaleIn(tween(200))) togetherWith (fadeOut(tween(200)) + scaleOut(
                        tween(200)
                    ))
                }, label = "theme_icon_animation"
            ) { processing ->
                if (processing) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = MaterialTheme.colorScheme.primary,
                        strokeWidth = 2.dp
                    )
                } else if (isSelected) {
                    Icon(
                        imageVector = Icons.Filled.Check,
                        contentDescription = "Selected",
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(24.dp)
                    )
                }
            }
        }
    }
}