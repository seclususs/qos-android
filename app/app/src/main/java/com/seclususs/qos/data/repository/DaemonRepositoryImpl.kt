package com.seclususs.qos.data.repository

import android.content.Context
import com.seclususs.qos.R
import com.seclususs.qos.core.di.IoDispatcher
import com.seclususs.qos.data.local.root.RootShell
import com.seclususs.qos.domain.model.DaemonMetrics
import com.seclususs.qos.domain.repository.DaemonRepository
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DaemonRepositoryImpl @Inject constructor(
    private val rootShell: RootShell,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher
) : DaemonRepository {

    companion object {
        private const val DAEMON_BIN = "/data/adb/modules/sys_qos/system/bin/qos_daemon"
        private const val PID_FILE = "/data/adb/modules/sys_qos/daemon.pid"
        private const val SERVICE_SCRIPT = "/data/adb/modules/sys_qos/service.sh"
        private var hasAttemptedAutoFix = false
    }

    override suspend fun checkDaemonExists(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("test -f $DAEMON_BIN")
    }

    override suspend fun getDaemonPid(): String? = withContext(ioDispatcher) {
        val script = """
            if [ -f $PID_FILE ]; then
                if kill -0 `cat $PID_FILE` 2>/dev/null; then
                    cat $PID_FILE
                else
                    rm -f $PID_FILE
                fi
            fi
        """.trimIndent()
        return@withContext rootShell.execute(script)?.trim()
    }

    override suspend fun startDaemon(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("sh $SERVICE_SCRIPT")
        return@withContext true
    }

    override suspend fun stopDaemon(): Boolean = withContext(ioDispatcher) {
        val script = """
            if [ -f $PID_FILE ]; then
                kill -9 `cat $PID_FILE` 2>/dev/null
            fi
            killall -9 qos_daemon 2>/dev/null
            rm -f $PID_FILE
        """.trimIndent()
        rootShell.executeSilently(script)
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
            val parts = result.trim().split(Regex("\\s+"))
            if (parts.size >= 4) {
                val cpu = "${parts[0]}%"
                val ramKb = parts[1].toLongOrNull() ?: 0L
                val ramMb = ramKb / 1024
                val ramPercent = parts[2]
                val ram = "${ramMb}MB ($ramPercent%)"
                val rawUptime = parts[3]
                return@withContext DaemonMetrics(
                    cpuUsage = cpu, ramUsage = ram, uptime = formatUptime(rawUptime)
                )
            }
        } catch (_: Exception) {
        }

        return@withContext DaemonMetrics()
    }

    private fun formatUptime(raw: String): String {
        return try {
            if (raw.contains("-")) {
                val parts = raw.split("-")
                context.getString(R.string.metric_uptime_days, parts[0], parts[1])
            } else if (raw.count { it == ':' } == 1) {
                "00:$raw"
            } else {
                raw
            }
        } catch (_: Exception) {
            raw
        }
    }
}