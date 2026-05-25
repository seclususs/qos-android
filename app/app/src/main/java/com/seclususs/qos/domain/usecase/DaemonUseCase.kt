package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import javax.inject.Inject

class DaemonUseCase @Inject constructor(
    private val repository: DaemonRepository
) {
    suspend fun checkExists(): Boolean {
        return repository.checkDaemonExists()
    }

    suspend fun getPid(): String {
        return repository.getDaemonPid() ?: "-"
    }

    suspend fun getMetrics(pid: String): DaemonMetrics {
        if (pid == "-") return DaemonMetrics()
        return repository.getDaemonMetrics(pid)
    }

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