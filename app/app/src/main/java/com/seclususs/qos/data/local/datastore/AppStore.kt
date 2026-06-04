package com.seclususs.qos.data.local.datastore

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.seclususs.qos.domain.repository.PreferencesRepository
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

private val Context.dataStore by preferencesDataStore(name = "qos_preferences")

@Singleton
class AppStore @Inject constructor(
    @param:ApplicationContext private val context: Context
) : PreferencesRepository {
    private val dataStore = context.dataStore

    companion object {
        val CACHED_CPU_LIMITS_KEY = stringPreferencesKey("cached_cpu_limits")
        val CACHED_STORAGE_LIMITS_KEY = stringPreferencesKey("cached_storage_limits")
        val NEEDS_REBOOT_KEY = booleanPreferencesKey("needs_reboot")
    }

    override val cachedCpuLimitsFlow: Flow<String> =
        dataStore.data.map { it[CACHED_CPU_LIMITS_KEY] ?: "" }

    override val cachedStorageLimitsFlow: Flow<String> =
        dataStore.data.map { it[CACHED_STORAGE_LIMITS_KEY] ?: "" }

    override val needsRebootFlow: Flow<Boolean> =
        dataStore.data.map { it[NEEDS_REBOOT_KEY] ?: false }

    override suspend fun setCachedCpuLimits(data: String) {
        dataStore.edit { it[CACHED_CPU_LIMITS_KEY] = data }
    }

    override suspend fun setCachedStorageLimits(data: String) {
        dataStore.edit { it[CACHED_STORAGE_LIMITS_KEY] = data }
    }

    override suspend fun setNeedsReboot(needs: Boolean) {
        dataStore.edit { it[NEEDS_REBOOT_KEY] = needs }
    }
}