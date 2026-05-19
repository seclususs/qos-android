package com.seclususs.qos.data.local

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

private val Context.dataStore by preferencesDataStore(name = "qos_preferences")

enum class AppTheme {
    LIGHT, DARK, SYSTEM
}

@Singleton
class AppStore @Inject constructor(
    @param:ApplicationContext private val context: Context
) {
    private val dataStore = context.dataStore

    companion object {
        val APP_THEME_KEY = stringPreferencesKey("app_theme")
        val CACHED_CPU_LIMITS_KEY = stringPreferencesKey("cached_cpu_limits")
        val CACHED_STORAGE_LIMITS_KEY = stringPreferencesKey("cached_storage_limits")
    }

    val appThemeFlow: Flow<AppTheme> = dataStore.data.map { preferences ->
        val themeString = preferences[APP_THEME_KEY] ?: AppTheme.SYSTEM.name
        try {
            AppTheme.valueOf(themeString)
        } catch (_: IllegalArgumentException) {
            AppTheme.SYSTEM
        }
    }

    val cachedCpuLimitsFlow: Flow<String> = dataStore.data.map { preferences ->
        preferences[CACHED_CPU_LIMITS_KEY] ?: ""
    }

    val cachedStorageLimitsFlow: Flow<String> = dataStore.data.map { preferences ->
        preferences[CACHED_STORAGE_LIMITS_KEY] ?: ""
    }

    suspend fun setAppTheme(theme: AppTheme) {
        dataStore.edit { preferences ->
            preferences[APP_THEME_KEY] = theme.name
        }
    }

    suspend fun setCachedCpuLimits(data: String) {
        dataStore.edit { preferences ->
            preferences[CACHED_CPU_LIMITS_KEY] = data
        }
    }

    suspend fun setCachedStorageLimits(data: String) {
        dataStore.edit { preferences ->
            preferences[CACHED_STORAGE_LIMITS_KEY] = data
        }
    }
}