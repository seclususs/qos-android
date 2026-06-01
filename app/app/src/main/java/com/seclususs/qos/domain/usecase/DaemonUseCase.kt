package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import javax.inject.Inject

class DaemonUseCase @Inject constructor(
    private val repository: DaemonRepository
) {
    suspend fun checkExists(): Boolean = repository.checkDaemonExists()
    suspend fun getPid(): String = repository.getDaemonPid() ?: "-"
    suspend fun getMetrics(pid: String): DaemonMetrics =
        if (pid == "-") DaemonMetrics() else repository.getDaemonMetrics(pid)

    suspend fun start(): Boolean = repository.startDaemon()
    suspend fun stop(): Boolean = repository.stopDaemon()
    suspend fun restart(): Boolean = repository.restartDaemon()
}