package com.seclususs.qos.ui.features.settings

import com.seclususs.qos.data.local.AppTheme

data class SettingsState(
    val appTheme: AppTheme = AppTheme.SYSTEM,
    val processingTheme: AppTheme? = null,
    val showThemeSheet: Boolean = false,
    val showDeveloperSheet: Boolean = false,
    val snackbarMessageResId: Int? = null,
    val snackbarIsError: Boolean = false,
    val snackbarVisible: Boolean = false
)

sealed interface SettingsEvent {
    data object OnThemeCardClicked : SettingsEvent
    data object OnDismissThemeSheet : SettingsEvent
    data class OnThemeSelected(val theme: AppTheme) : SettingsEvent
    data object OnDeveloperCardClicked : SettingsEvent
    data object OnDismissDeveloperSheet : SettingsEvent
    data object OnDismissSnackbar : SettingsEvent
}