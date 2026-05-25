package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.repository.DaemonRepository
import javax.inject.Inject

class ToggleDaemonUseCase @Inject constructor(
    private val repository: DaemonRepository
) {
    suspend fun start(): Boolean {
        return repository.startDaemon()
    }

    suspend fun stop(): Boolean {
        return repository.stopDaemon()
    }

    suspend fun restart(): Boolean {
        return repository.restartDaemon()
    }
}