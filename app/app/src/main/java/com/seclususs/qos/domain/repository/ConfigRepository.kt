package com.seclususs.qos.domain.repository

import com.seclususs.qos.domain.model.QosConfig

interface ConfigRepository {
    suspend fun checkConfigExists(): Boolean
    suspend fun getConfig(): QosConfig
    suspend fun updateConfig(config: QosConfig): Boolean
}