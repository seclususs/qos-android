package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.model.QosConfig
import com.seclususs.qos.domain.repository.ConfigRepository
import javax.inject.Inject

class GetConfigUseCase @Inject constructor(
    private val repository: ConfigRepository
) {
    suspend operator fun invoke(): QosConfig {
        return repository.getConfig()
    }
}