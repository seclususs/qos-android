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
            ; ================================================
            ; Configuration
            ; ================================================
            
            ; [Blocker Controller]
            blocker_enabled=${config.blockerEnabled}
            
            ; [Cleaner Controller]
            cleaner_enabled=${config.cleanerEnabled}
            
            ; [CPU Controller]
            cpu_enabled=${config.cpuEnabled}
            
            ; [Storage Controller]
            storage_enabled=${config.storageEnabled}
            
            ; [System Tweaks]
            tweaks_enabled=${config.tweaksEnabled}
            
            ; ================================================
            ; Advanced Tuning
            ; ================================================
            
            ; [CPU Kernel Limits]
            min_latency_ns=${config.minLatencyNs ?: ""}
            max_latency_ns=${config.maxLatencyNs ?: ""}
            min_granularity_ns=${config.minGranularityNs ?: ""}
            max_granularity_ns=${config.maxGranularityNs ?: ""}
            min_wakeup_ns=${config.minWakeupNs ?: ""}
            max_wakeup_ns=${config.maxWakeupNs ?: ""}
            min_migration_cost=${config.minMigrationCost ?: ""}
            max_migration_cost=${config.maxMigrationCost ?: ""}
            min_walt_init_pct=${config.minWaltInitPct ?: ""}
            max_walt_init_pct=${config.maxWaltInitPct ?: ""}
            min_uclamp_min=${config.minUclampMin ?: ""}
            max_uclamp_min=${config.maxUclampMin ?: ""}
            
            ; [Storage Kernel Limits]
            min_read_ahead=${config.minReadAhead ?: ""}
            max_read_ahead=${config.maxReadAhead ?: ""}
            min_nr_requests=${config.minNrRequests ?: ""}
            max_nr_requests=${config.maxNrRequests ?: ""}
        """.trimIndent()
    }
}