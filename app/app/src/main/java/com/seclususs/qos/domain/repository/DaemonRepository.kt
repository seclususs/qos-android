package com.seclususs.qos.domain.repository

import com.seclususs.qos.domain.model.DaemonMetrics

interface DaemonRepository {
    suspend fun checkDaemonExists(): Boolean
    suspend fun getDaemonPid(): String?
    suspend fun startDaemon(): Boolean
    suspend fun stopDaemon(): Boolean
    suspend fun restartDaemon(): Boolean
    suspend fun getDaemonMetrics(pid: String): DaemonMetrics
}