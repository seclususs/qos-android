package com.seclususs.qos.data.repository

import com.seclususs.qos.data.local.AppStore
import com.seclususs.qos.data.local.AppTheme
import com.seclususs.qos.domain.repository.AppPreferencesRepository
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AppPreferencesRepositoryImpl @Inject constructor(
    private val appStore: AppStore
) : AppPreferencesRepository {

    override val appThemeFlow: Flow<AppTheme> = appStore.appThemeFlow
    override val cachedCpuLimitsFlow: Flow<String> = appStore.cachedCpuLimitsFlow
    override val cachedStorageLimitsFlow: Flow<String> = appStore.cachedStorageLimitsFlow

    override suspend fun setAppTheme(theme: AppTheme) {
        appStore.setAppTheme(theme)
    }

    override suspend fun setCachedCpuLimits(data: String) {
        appStore.setCachedCpuLimits(data)
    }

    override suspend fun setCachedStorageLimits(data: String) {
        appStore.setCachedStorageLimits(data)
    }
}