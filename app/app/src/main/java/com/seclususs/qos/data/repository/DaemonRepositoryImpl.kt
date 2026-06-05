package com.seclususs.qos.data.repository

import android.content.Context
import com.seclususs.qos.R
import com.seclususs.qos.core.di.IoDispatcher
import com.seclususs.qos.data.local.root.RootShell
import com.seclususs.qos.domain.model.DaemonInfo
import com.seclususs.qos.domain.repository.DaemonRepository
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import java.util.Locale
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
        private val SPACE_REGEX = Regex("\\s+")
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

    override suspend fun getDaemonInfo(pid: String): DaemonInfo = withContext(ioDispatcher) {
        if (pid.isBlank() || pid == "-") return@withContext DaemonInfo(pid = pid)

        val cmd =
            "cat /proc/$pid/status 2>/dev/null; echo '=='; cat /proc/meminfo 2>/dev/null; echo '=='; cat /proc/$pid/stat 2>/dev/null; echo '=='; cat /proc/uptime 2>/dev/null"
        val out =
            rootShell.execute(cmd)?.split("==")?.map { it.trim() } ?: return@withContext DaemonInfo(
                pid = pid
            )

        if (out.size < 4) return@withContext DaemonInfo(pid = pid)
        return@withContext runCatching {
            val stat = out[2].substringAfter(") ").trim().split(SPACE_REGEX)
            val uptimeSec = out[3].substringBefore(" ").toDoubleOrNull() ?: 0.0
            val startTimeTicks = stat.getOrNull(19)?.toDoubleOrNull() ?: 0.0
            val procSec = uptimeSec - (startTimeTicks / 100.0)

            if (procSec > 0) {
                val days = (procSec / 86400).toInt()
                val hours = ((procSec % 86400) / 3600).toInt()
                val minutes = ((procSec % 3600) / 60).toInt()
                val secs = (procSec % 60).toInt()
                val timeString = String.format(Locale.US, "%02d:%02d:%02d", hours, minutes, secs)
                val uptime = if (days > 0) {
                    context.getString(R.string.metric_uptime_days, days.toString(), timeString)
                } else {
                    timeString
                }
                DaemonInfo(uptime, pid)
            } else {
                DaemonInfo(pid = pid)
            }
        }.getOrDefault(DaemonInfo(pid = pid))
    }

    override suspend fun stopDaemon(): Boolean = withContext(ioDispatcher) {
        val script = """
            if [ -f $PID_FILE ]; then
                kill -15 `cat $PID_FILE` 2>/dev/null
                sleep 0.2
                kill -9 `cat $PID_FILE` 2>/dev/null
            fi
            killall -15 qos_daemon 2>/dev/null
            sleep 0.1
            killall -9 qos_daemon 2>/dev/null
            rm -f $PID_FILE
        """.trimIndent()
        rootShell.executeSilently(script)
        return@withContext true
    }

    override suspend fun rebootDevice(): Boolean = withContext(ioDispatcher) {
        rootShell.executeSilently("su -c svc power reboot || reboot")
        return@withContext true
    }
}