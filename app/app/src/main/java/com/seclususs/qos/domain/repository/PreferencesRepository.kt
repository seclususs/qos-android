package com.seclususs.qos.domain.repository

import kotlinx.coroutines.flow.Flow

interface PreferencesRepository {
    val advancedCpuEnabledFlow: Flow<Boolean>
    val advancedStorageEnabledFlow: Flow<Boolean>
    val cachedCpuLimitsFlow: Flow<String>
    val cachedStorageLimitsFlow: Flow<String>
    val needsRebootFlow: Flow<Boolean>
    suspend fun setAdvancedCpuEnabled(enabled: Boolean)
    suspend fun setAdvancedStorageEnabled(enabled: Boolean)
    suspend fun setCachedCpuLimits(data: String)
    suspend fun setCachedStorageLimits(data: String)
    suspend fun setNeedsReboot(needs: Boolean)
}