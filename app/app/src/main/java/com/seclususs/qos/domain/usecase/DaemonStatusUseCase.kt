package com.seclususs.qos.domain.usecase

import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import javax.inject.Inject

class DaemonStatusUseCase @Inject constructor(
    private val repository: DaemonRepository
) {
    suspend fun isRunning(): Boolean {
        return repository.isDaemonRunning()
    }

    suspend fun getPid(): String {
        return repository.getDaemonPid() ?: "-"
    }

    suspend fun getMetrics(pid: String): DaemonMetrics {
        if (pid == "-") return DaemonMetrics()
        return repository.getDaemonMetrics(pid)
    }
}