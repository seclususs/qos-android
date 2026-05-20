package com.seclususs.qos.domain.repository

import com.seclususs.qos.data.local.AppTheme
import kotlinx.coroutines.flow.Flow

interface AppPreferencesRepository {
    val appThemeFlow: Flow<AppTheme>
    val cachedCpuLimitsFlow: Flow<String>
    val cachedStorageLimitsFlow: Flow<String>

    suspend fun setAppTheme(theme: AppTheme)
    suspend fun setCachedCpuLimits(data: String)
    suspend fun setCachedStorageLimits(data: String)
}