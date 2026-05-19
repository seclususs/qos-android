package com.seclususs.qos.domain.model

data class QosConfig(
    val blockerEnabled: Boolean = true,
    val cleanerEnabled: Boolean = true,
    val cpuEnabled: Boolean = true,
    val storageEnabled: Boolean = true,
    val tweaksEnabled: Boolean = true,
    val minLatencyNs: Long? = 8_000_000L,
    val maxLatencyNs: Long? = 20_000_000L,
    val minGranularityNs: Long? = 2_500_000L,
    val maxGranularityNs: Long? = 6_500_000L,
    val minWakeupNs: Long? = 1_500_000L,
    val maxWakeupNs: Long? = 6_500_000L,
    val minMigrationCost: Long? = 200_000L,
    val maxMigrationCost: Long? = 600_000L,
    val minWaltInitPct: Long? = 10L,
    val maxWaltInitPct: Long? = 40L,
    val minUclampMin: Long? = 0L,
    val maxUclampMin: Long? = 384L,
    val minReadAhead: Long? = 128L,
    val maxReadAhead: Long? = 1024L,
    val minNrRequests: Long? = 64L,
    val maxNrRequests: Long? = 256L
)