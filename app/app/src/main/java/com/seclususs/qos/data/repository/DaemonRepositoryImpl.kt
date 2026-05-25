package com.seclususs.qos.data.repository

import com.seclususs.qos.core.di.IoDispatcher
import com.seclususs.qos.data.local.root.RootShell
import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DaemonRepositoryImpl @Inject constructor(
    private val rootShell: RootShell,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher
) : DaemonRepository {

    companion object {
        private val SPACE_REGEX = Regex("\\s+")
        private const val PID_FILE = "/data/adb/modules/sys_qos/daemon.pid"
        private var hasAttemptedAutoFix = false
    }

    override suspend fun checkDaemonExists(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("ls /data/adb/modules/sys_qos/system/bin/qos_daemon")
    }

    override suspend fun getDaemonPid(): String? = withContext(ioDispatcher) {
        val pid = rootShell.readFile(PID_FILE)?.trim()

        if (!pid.isNullOrBlank() && pid.all { it.isDigit() }) {
            if (rootShell.executeSilently("kill -0 $pid")) {
                return@withContext pid
            } else {
                rootShell.executeSilently("rm -f $PID_FILE")
            }
        }

        return@withContext null
    }

    override suspend fun startDaemon(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("nohup sh /data/adb/modules/sys_qos/service.sh > /dev/null 2>&1 &")
        return@withContext true
    }

    override suspend fun stopDaemon(): Boolean = withContext(ioDispatcher) {
        val pid = getDaemonPid()

        if (!pid.isNullOrBlank()) {
            rootShell.executeSilently("kill -9 $pid")
        }

        rootShell.executeSilently("killall -9 qos_daemon")
        rootShell.executeSilently("rm -f $PID_FILE")
        return@withContext true
    }

    override suspend fun restartDaemon(): Boolean = withContext(ioDispatcher) {
        stopDaemon()
        delay(500)
        startDaemon()
        return@withContext true
    }

    override suspend fun getDaemonMetrics(pid: String): DaemonMetrics = withContext(ioDispatcher) {
        if (pid.isBlank() || pid == "-") return@withContext DaemonMetrics()

        val result = rootShell.execute("ps -o %cpu,rss,%mem,etime -p $pid | tail -n 1")
        if (result.isNullOrBlank() || result.contains("ELAPSED") || result.contains("%CPU")) {
            if (!hasAttemptedAutoFix) {
                hasAttemptedAutoFix = true
                restartDaemon()
            }
            return@withContext DaemonMetrics()
        }

        hasAttemptedAutoFix = false

        try {
            val parts = result.trim().split(SPACE_REGEX)
            if (parts.size >= 4) {
                val cpu = "${parts[0]}%"
                val ramKb = parts[1].toLongOrNull() ?: 0L
                val ramMb = ramKb / 1024
                val ramPercent = parts[2]
                val ram = "${ramMb}MB ($ramPercent%)"
                val uptime = parts[3]
                return@withContext DaemonMetrics(
                    cpuUsage = cpu, ramUsage = ram, uptime = uptime
                )
            }
        } catch (_: Exception) {
        }

        return@withContext DaemonMetrics()
    }
}