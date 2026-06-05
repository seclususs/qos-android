package com.seclususs.qos.domain.model

data class QosConfig(
    val blockerEnabled: Boolean = true,
    val cleanerEnabled: Boolean = true,
    val cpuEnabled: Boolean = true,
    val storageEnabled: Boolean = true,
    val tweaksEnabled: Boolean = true,
    val minLatencyNs: Long? = null,
    val maxLatencyNs: Long? = null,
    val minGranularityNs: Long? = null,
    val maxGranularityNs: Long? = null,
    val minWakeupNs: Long? = null,
    val maxWakeupNs: Long? = null,
    val minMigrationCost: Long? = null,
    val maxMigrationCost: Long? = null,
    val minWaltInitPct: Long? = null,
    val maxWaltInitPct: Long? = null,
    val minUclampMin: Long? = null,
    val maxUclampMin: Long? = null,
    val minReadAhead: Long? = null,
    val maxReadAhead: Long? = null,
    val minNrRequests: Long? = null,
    val maxNrRequests: Long? = null
)

fun Map<String, String>.getLong(key: String, default: Long? = null): Long? =
    this[key]?.toLongOrNull() ?: default

fun Map<String, String>.getUpdatedLong(key: String, current: Long?): Long? =
    if (this.containsKey(key)) this[key]?.toLongOrNull() else current

fun Map<String, String>.encodeToString(): String =
    entries.joinToString(";") { "${it.key}=${it.value}" }

fun String.decodeToMap(): Map<String, String> {
    if (this.isBlank()) return emptyMap()
    return splitToSequence(';').mapNotNull { pair ->
        val index = pair.indexOf('=')
        if (index != -1) pair.substring(0, index) to pair.substring(index + 1) else null
    }.toMap()
}

fun QosConfig.applyCpuLimits(cached: Map<String, String>): QosConfig {
    return copy(
        minLatencyNs = cached.getLong(ConfigKeys.CPU_LATENCY_MIN, null),
        maxLatencyNs = cached.getLong(ConfigKeys.CPU_LATENCY_MAX, null),
        minGranularityNs = cached.getLong(ConfigKeys.CPU_GRANULARITY_MIN, null),
        maxGranularityNs = cached.getLong(ConfigKeys.CPU_GRANULARITY_MAX, null),
        minWakeupNs = cached.getLong(ConfigKeys.CPU_WAKEUP_MIN, null),
        maxWakeupNs = cached.getLong(ConfigKeys.CPU_WAKEUP_MAX, null),
        minMigrationCost = cached.getLong(ConfigKeys.CPU_MIGRATION_COST_MIN, null),
        maxMigrationCost = cached.getLong(ConfigKeys.CPU_MIGRATION_COST_MAX, null),
        minWaltInitPct = cached.getLong(ConfigKeys.CPU_WALT_INIT_MIN, null),
        maxWaltInitPct = cached.getLong(ConfigKeys.CPU_WALT_INIT_MAX, null),
        minUclampMin = cached.getLong(ConfigKeys.CPU_UCLAMP_MIN_MIN, null),
        maxUclampMin = cached.getLong(ConfigKeys.CPU_UCLAMP_MIN_MAX, null)
    )
}

fun QosConfig.clearCpuLimits(): QosConfig = copy(
    minLatencyNs = null,
    maxLatencyNs = null,
    minGranularityNs = null,
    maxGranularityNs = null,
    minWakeupNs = null,
    maxWakeupNs = null,
    minMigrationCost = null,
    maxMigrationCost = null,
    minWaltInitPct = null,
    maxWaltInitPct = null,
    minUclampMin = null,
    maxUclampMin = null
)

fun QosConfig.updateCpuLimits(cpuValues: Map<String, String>): QosConfig = copy(
    minLatencyNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_LATENCY_MIN, minLatencyNs),
    maxLatencyNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_LATENCY_MAX, maxLatencyNs),
    minGranularityNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_GRANULARITY_MIN, minGranularityNs),
    maxGranularityNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_GRANULARITY_MAX, maxGranularityNs),
    minWakeupNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_WAKEUP_MIN, minWakeupNs),
    maxWakeupNs = cpuValues.getUpdatedLong(ConfigKeys.CPU_WAKEUP_MAX, maxWakeupNs),
    minMigrationCost = cpuValues.getUpdatedLong(
        ConfigKeys.CPU_MIGRATION_COST_MIN, minMigrationCost
    ),
    maxMigrationCost = cpuValues.getUpdatedLong(
        ConfigKeys.CPU_MIGRATION_COST_MAX, maxMigrationCost
    ),
    minWaltInitPct = cpuValues.getUpdatedLong(ConfigKeys.CPU_WALT_INIT_MIN, minWaltInitPct),
    maxWaltInitPct = cpuValues.getUpdatedLong(ConfigKeys.CPU_WALT_INIT_MAX, maxWaltInitPct),
    minUclampMin = cpuValues.getUpdatedLong(ConfigKeys.CPU_UCLAMP_MIN_MIN, minUclampMin),
    maxUclampMin = cpuValues.getUpdatedLong(ConfigKeys.CPU_UCLAMP_MIN_MAX, maxUclampMin)
)

fun QosConfig.applyStorageLimits(cached: Map<String, String>): QosConfig {
    return copy(
        minReadAhead = cached.getLong(ConfigKeys.STORAGE_READ_AHEAD_MIN, null),
        maxReadAhead = cached.getLong(ConfigKeys.STORAGE_READ_AHEAD_MAX, null),
        minNrRequests = cached.getLong(ConfigKeys.STORAGE_NR_REQUESTS_MIN, null),
        maxNrRequests = cached.getLong(ConfigKeys.STORAGE_NR_REQUESTS_MAX, null)
    )
}

fun QosConfig.clearStorageLimits(): QosConfig = copy(
    minReadAhead = null, maxReadAhead = null, minNrRequests = null, maxNrRequests = null
)

fun QosConfig.updateStorageLimits(storageValues: Map<String, String>): QosConfig = copy(
    minReadAhead = storageValues.getUpdatedLong(ConfigKeys.STORAGE_READ_AHEAD_MIN, minReadAhead),
    maxReadAhead = storageValues.getUpdatedLong(ConfigKeys.STORAGE_READ_AHEAD_MAX, maxReadAhead),
    minNrRequests = storageValues.getUpdatedLong(ConfigKeys.STORAGE_NR_REQUESTS_MIN, minNrRequests),
    maxNrRequests = storageValues.getUpdatedLong(ConfigKeys.STORAGE_NR_REQUESTS_MAX, maxNrRequests)
)