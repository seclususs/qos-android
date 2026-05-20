package com.seclususs.qos.ui.features.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.seclususs.qos.R
import com.seclususs.qos.data.local.AppStore
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
    private val appStore: AppStore
) : ViewModel() {

    private val _state = MutableStateFlow(SettingsState())
    val state: StateFlow<SettingsState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            appStore.appThemeFlow.collect { theme ->
                _state.update { it.copy(appTheme = theme) }
            }
        }
    }

    fun onEvent(event: SettingsEvent) {
        when (event) {
            is SettingsEvent.OnThemeCardClicked -> {
                _state.update { it.copy(showThemeSheet = true) }
            }

            is SettingsEvent.OnDismissThemeSheet -> {
                _state.update { it.copy(showThemeSheet = false) }
            }

            is SettingsEvent.OnThemeSelected -> {
                viewModelScope.launch {
                    _state.update { it.copy(processingTheme = event.theme) }

                    delay(600)

                    try {
                        appStore.setAppTheme(event.theme)

                        _state.update {
                            it.copy(
                                processingTheme = null,
                                snackbarMessageResId = R.string.theme_change_success,
                                snackbarIsError = false,
                                snackbarVisible = true
                            )
                        }
                    } catch (_: Exception) {
                        _state.update {
                            it.copy(
                                processingTheme = null,
                                snackbarMessageResId = R.string.theme_change_error,
                                snackbarIsError = true,
                                snackbarVisible = true
                            )
                        }
                    }

                    delay(3000)
                    _state.update { it.copy(snackbarVisible = false) }
                }
            }

            is SettingsEvent.OnDeveloperCardClicked -> {
                _state.update { it.copy(showDeveloperSheet = true) }
            }

            is SettingsEvent.OnDismissDeveloperSheet -> {
                _state.update { it.copy(showDeveloperSheet = false) }
            }

            is SettingsEvent.OnDismissSnackbar -> {
                _state.update { it.copy(snackbarVisible = false) }
            }
        }
    }
}