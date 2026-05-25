package com.seclususs.qos.ui.features.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.R
import com.seclususs.qos.data.local.AppTheme
import com.seclususs.qos.domain.repository.AppPreferencesRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val appPreferencesRepository: AppPreferencesRepository
) : ViewModel() {

    private val _state = MutableStateFlow(SettingsState())
    val state: StateFlow<SettingsState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            appPreferencesRepository.appThemeFlow.collect { theme ->
                _state.update { it.copy(appTheme = theme) }
            }
        }
    }

    fun onEvent(event: SettingsEvent) {
        when (event) {
            is SettingsEvent.OnThemeCardClicked -> _state.update { it.copy(showThemeSheet = true) }
            is SettingsEvent.OnDismissThemeSheet -> _state.update { it.copy(showThemeSheet = false) }
            is SettingsEvent.OnDeveloperCardClicked -> _state.update { it.copy(showDeveloperSheet = true) }
            is SettingsEvent.OnDismissDeveloperSheet -> _state.update { it.copy(showDeveloperSheet = false) }
            is SettingsEvent.OnDismissSnackbar -> _state.update { it.copy(snackbarVisible = false) }
            is SettingsEvent.OnThemeSelected -> updateTheme(event.theme)
        }
    }

    private fun updateTheme(theme: AppTheme) {
        viewModelScope.launch {
            _state.update { it.copy(processingTheme = theme) }
            delay(400)
            try {
                appPreferencesRepository.setAppTheme(theme)
                showSnackbar(R.string.theme_change_success, isError = false)
            } catch (_: Exception) {
                showSnackbar(R.string.theme_change_error, isError = true)
            } finally {
                _state.update { it.copy(processingTheme = null) }
            }
        }
    }

    private suspend fun showSnackbar(messageRes: Int, isError: Boolean) {
        _state.update {
            it.copy(
                snackbarMessageResId = messageRes, snackbarIsError = isError, snackbarVisible = true
            )
        }
        delay(3000)
        _state.update { it.copy(snackbarVisible = false) }
    }
}