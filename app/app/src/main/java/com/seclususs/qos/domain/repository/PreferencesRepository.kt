package com.seclususs.qos.domain.repository

import kotlinx.coroutines.flow.Flow

interface PreferencesRepository {
    val cachedCpuLimitsFlow: Flow<String>
    val cachedStorageLimitsFlow: Flow<String>
    suspend fun setCachedCpuLimits(data: String)
    suspend fun setCachedStorageLimits(data: String)
}