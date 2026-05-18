package com.seclususs.qos.data.repository

import com.seclususs.qos.core.di.IoDispatcher
import com.seclususs.qos.data.local.root.RootShell
import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DaemonRepositoryImpl @Inject constructor(
    private val rootShell: RootShell,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher
) : DaemonRepository {

    override suspend fun isDaemonRunning(): Boolean = withContext(ioDispatcher) {
        val pid = rootShell.execute("pidof qos_daemon")
        !pid.isNullOrBlank()
    }

    override suspend fun getDaemonPid(): String? = withContext(ioDispatcher) {
        val pid = rootShell.execute("pidof qos_daemon")
        if (pid.isNullOrBlank()) null else pid.trim()
    }

    override suspend fun startDaemon(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("/data/adb/modules/sys_qos/system/bin/qos_daemon &")
    }

    override suspend fun stopDaemon(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("killall qos_daemon")
    }

    override suspend fun getDaemonMetrics(pid: String): DaemonMetrics = withContext(ioDispatcher) {
        if (pid.isBlank() || pid == "-") return@withContext DaemonMetrics()

        val result = rootShell.execute("ps -o %cpu,rss,etime -p $pid | tail -n 1")

        if (result.isNullOrBlank()) return@withContext DaemonMetrics()

        try {
            val parts = result.trim().split(Regex("\\s+"))
            if (parts.size >= 3) {
                val cpu = "${parts[0]}%"
                val ramKb = parts[1].toLongOrNull() ?: 0L
                val ramMb = ramKb / 1024
                val ram = "${ramMb}MB"
                val uptime = parts[2]
                return@withContext DaemonMetrics(
                    cpuUsage = cpu, ramUsage = ram, uptime = uptime
                )
            }
        } catch (_: Exception) {
        }

        return@withContext DaemonMetrics()
    }
}