package com.seclususs.qos.data.local.root

import com.seclususs.qos.domain.model.QosConfig
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ConfigParser @Inject constructor() {

    fun parse(rawIniText: String): QosConfig {
        return rawIniText.lineSequence().map { it.trim() }
            .filter { it.isNotEmpty() && !it.startsWith(";") && !it.startsWith("#") }
            .fold(QosConfig()) { currentConfig, line ->
                val index = line.indexOf('=')
                if (index != -1) {
                    val key = line.substring(0, index).trim()
                    val value = line.substring(index + 1).trim()
                    updateConfigValue(currentConfig, key, value)
                } else currentConfig
            }
    }

    private fun updateConfigValue(currentConfig: QosConfig, key: String, value: String): QosConfig {
        return try {
            when (key) {
                "blocker_enabled" -> currentConfig.copy(
                    blockerEnabled = value.toBooleanStrictOrNull() ?: currentConfig.blockerEnabled
                )

                "cleaner_enabled" -> currentConfig.copy(
                    cleanerEnabled = value.toBooleanStrictOrNull() ?: currentConfig.cleanerEnabled
                )

                "cpu_enabled" -> currentConfig.copy(
                    cpuEnabled = value.toBooleanStrictOrNull() ?: currentConfig.cpuEnabled
                )

                "storage_enabled" -> currentConfig.copy(
                    storageEnabled = value.toBooleanStrictOrNull() ?: currentConfig.storageEnabled
                )

                "tweaks_enabled" -> currentConfig.copy(
                    tweaksEnabled = value.toBooleanStrictOrNull() ?: currentConfig.tweaksEnabled
                )

                "min_latency_ns" -> currentConfig.copy(minLatencyNs = value.toLongOrNull())
                "max_latency_ns" -> currentConfig.copy(maxLatencyNs = value.toLongOrNull())
                "min_granularity_ns" -> currentConfig.copy(minGranularityNs = value.toLongOrNull())
                "max_granularity_ns" -> currentConfig.copy(maxGranularityNs = value.toLongOrNull())
                "min_wakeup_ns" -> currentConfig.copy(minWakeupNs = value.toLongOrNull())
                "max_wakeup_ns" -> currentConfig.copy(maxWakeupNs = value.toLongOrNull())
                "min_migration_cost" -> currentConfig.copy(minMigrationCost = value.toLongOrNull())
                "max_migration_cost" -> currentConfig.copy(maxMigrationCost = value.toLongOrNull())
                "min_walt_init_pct" -> currentConfig.copy(minWaltInitPct = value.toLongOrNull())
                "max_walt_init_pct" -> currentConfig.copy(maxWaltInitPct = value.toLongOrNull())
                "min_uclamp_min" -> currentConfig.copy(minUclampMin = value.toLongOrNull())
                "max_uclamp_min" -> currentConfig.copy(maxUclampMin = value.toLongOrNull())
                "min_read_ahead" -> currentConfig.copy(minReadAhead = value.toLongOrNull())
                "max_read_ahead" -> currentConfig.copy(maxReadAhead = value.toLongOrNull())
                "min_nr_requests" -> currentConfig.copy(minNrRequests = value.toLongOrNull())
                "max_nr_requests" -> currentConfig.copy(maxNrRequests = value.toLongOrNull())
                else -> currentConfig
            }
        } catch (_: Exception) {
            currentConfig
        }
    }

    fun serialize(config: QosConfig): String {
        return """
            ; ##############################################################################
            ; QOS Daemon Configuration
            ; ##############################################################################
            ; This file controls the core features of the daemon.
            ; Use 'true' to enable a feature, or 'false' to disable it.

            ; ##############################################################################
            ; [Blocker Controller]
            ; ##############################################################################
            ; Automatically blocks unnecessary background services (like GMS analytics 
            ; and ad trackers) to save battery and reduce random system wakeups.
            ;
            blocker_enabled=${config.blockerEnabled}

            ; ##############################################################################
            ; [Cleaner Controller]
            ; ##############################################################################
            ; Silently clears junk and stale cache in the background when your device 
            ; is idle. It adapts to thermal and CPU load to ensure zero stutters on 
            ; your active applications.
            ;
            cleaner_enabled=${config.cleanerEnabled}

            ; ##############################################################################
            ; [CPU Controller]
            ; ##############################################################################
            ; Actively monitors system pressure and tunes the CPU scheduler on the fly 
            ; to keep your device smooth while perfectly balancing battery life.
            ;
            cpu_enabled=${config.cpuEnabled}

            ; ##############################################################################
            ; [Storage Controller]
            ; ##############################################################################
            ; Optimizes storage read/write speeds by adjusting I/O queues dynamically. 
            ; This significantly reduces lag when opening heavy apps or loading large files.
            ;
            storage_enabled=${config.storageEnabled}

            ; ##############################################################################
            ; [System Tweaks]
            ; ##############################################################################
            ; Applies a curated set of kernel and Android properties to improve overall 
            ; UI responsiveness, network stability, and memory management.
            ;
            tweaks_enabled=${config.tweaksEnabled}


            ; ##############################################################################
            ; Advanced Tuning (Optional)
            ; ##############################################################################
            ; WARNING: The following parameters override the safe defaults baked into the
            ;          daemon. Modify these ONLY if you know exactly what you are doing.
            ;
            ; - The daemon will automatically clamp extreme values to prevent system crashes.
            ; - To use the safe default for a specific setting, simply leave it blank
            ;   or comment out the line using a semicolon (;).
            ; ##############################################################################

            ; ##############################################################################
            ; [CPU Kernel Limits]
            ; ##############################################################################
            ; Scheduler latency limits (in nanoseconds)
            ; e.g., min_latency_ns=10000000 (10ms) | max_latency_ns=20000000 (20ms)
            min_latency_ns=${config.minLatencyNs ?: ""}
            max_latency_ns=${config.maxLatencyNs ?: ""}

            ; Scheduler granularity limits (in nanoseconds)
            ; e.g., min_granularity_ns=2000000 (2ms) | max_granularity_ns=4000000 (4ms)
            min_granularity_ns=${config.minGranularityNs ?: ""}
            max_granularity_ns=${config.maxGranularityNs ?: ""}

            ; Wakeup limits (in nanoseconds)
            ; e.g., min_wakeup_ns=2000000 (2ms) | max_wakeup_ns=4000000 (4ms)
            min_wakeup_ns=${config.minWakeupNs ?: ""}
            max_wakeup_ns=${config.maxWakeupNs ?: ""}

            ; Task migration cost limits (in nanoseconds)
            ; e.g., min_migration_cost=250000 (0.25ms) | max_migration_cost=500000 (0.5ms)
            min_migration_cost=${config.minMigrationCost ?: ""}
            max_migration_cost=${config.maxMigrationCost ?: ""}

            ; Initial task load percentage for WALT scheduler (1 - 100)
            ; e.g., min_walt_init_pct=10 | max_walt_init_pct=25
            min_walt_init_pct=${config.minWaltInitPct ?: ""}
            max_walt_init_pct=${config.maxWaltInitPct ?: ""}

            ; Utilization clamp (UClamp) boundaries (0 - 1024)
            ; e.g., min_uclamp_min=0 | max_uclamp_min=200
            min_uclamp_min=${config.minUclampMin ?: ""}
            max_uclamp_min=${config.maxUclampMin ?: ""}

            ; ##############################################################################
            ; [Storage Kernel Limits]
            ; ##############################################################################
            ; Storage read-ahead size limits (in KB)
            ; e.g., min_read_ahead=128 | max_read_ahead=512
            min_read_ahead=${config.minReadAhead ?: ""}
            max_read_ahead=${config.maxReadAhead ?: ""}

            ; Storage I/O queue request limits
            ; e.g., min_nr_requests=64 | max_nr_requests=128
            min_nr_requests=${config.minNrRequests ?: ""}
            max_nr_requests=${config.maxNrRequests ?: ""}
        """.trimIndent()
    }
}