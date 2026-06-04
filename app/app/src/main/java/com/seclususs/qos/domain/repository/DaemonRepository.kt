package com.seclususs.qos.domain.repository

import com.seclususs.qos.domain.model.DaemonInfo

interface DaemonRepository {
    suspend fun checkDaemonExists(): Boolean
    suspend fun getDaemonPid(): String?
    suspend fun getDaemonInfo(pid: String): DaemonInfo
    suspend fun stopDaemon(): Boolean
    suspend fun rebootDevice(): Boolean
}