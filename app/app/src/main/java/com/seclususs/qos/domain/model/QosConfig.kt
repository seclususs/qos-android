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