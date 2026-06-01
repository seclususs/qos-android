package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.repository.ConfigRepository
import javax.inject.Inject

class ConfigUseCase @Inject constructor(
    private val repository: ConfigRepository
) {
    suspend fun checkExists(): Boolean = repository.checkConfigExists()
    suspend fun get(): QosConfig = repository.getConfig()
    suspend fun update(newConfig: QosConfig): Boolean = repository.updateConfig(newConfig)
}