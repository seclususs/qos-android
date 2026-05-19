package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.repository.ConfigRepository
import javax.inject.Inject

class CheckConfigUseCase @Inject constructor(
    private val repository: ConfigRepository
) {
    suspend operator fun invoke(): Boolean {
        return repository.checkConfigExists()
    }
}