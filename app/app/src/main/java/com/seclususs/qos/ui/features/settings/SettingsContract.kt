package com.seclususs.qos.ui.features.settings

import com.seclususs.qos.data.local.AppTheme

data class SettingsState(
    val appTheme: AppTheme = AppTheme.SYSTEM,
    val processingTheme: AppTheme? = null,
    val showThemeSheet: Boolean = false,
    val snackbarMessageResId: Int? = null,
    val snackbarIsError: Boolean = false,
    val snackbarVisible: Boolean = false
)

sealed class SettingsEvent {
    object OnThemeCardClicked : SettingsEvent()
    object OnDismissThemeSheet : SettingsEvent()
    data class OnThemeSelected(val theme: AppTheme) : SettingsEvent()
    object OnDismissSnackbar : SettingsEvent()
}