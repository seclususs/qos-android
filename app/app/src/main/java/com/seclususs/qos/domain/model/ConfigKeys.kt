package com.seclususs.qos.domain.model

object ConfigKeys {
    const val CPU_LATENCY_MIN = "latency_min"
    const val CPU_LATENCY_MAX = "latency_max"
    const val CPU_GRANULARITY_MIN = "granularity_min"
    const val CPU_GRANULARITY_MAX = "granularity_max"
    const val CPU_WAKEUP_MIN = "wakeup_min"
    const val CPU_WAKEUP_MAX = "wakeup_max"
    const val CPU_MIGRATION_COST_MIN = "migration_cost_min"
    const val CPU_MIGRATION_COST_MAX = "migration_cost_max"
    const val CPU_WALT_INIT_MIN = "walt_init_min"
    const val CPU_WALT_INIT_MAX = "walt_init_max"
    const val CPU_UCLAMP_MIN_MIN = "uclamp_min_min"
    const val CPU_UCLAMP_MIN_MAX = "uclamp_min_max"
    const val STORAGE_READ_AHEAD_MIN = "read_ahead_min"
    const val STORAGE_READ_AHEAD_MAX = "read_ahead_max"
    const val STORAGE_NR_REQUESTS_MIN = "nr_requests_min"
    const val STORAGE_NR_REQUESTS_MAX = "nr_requests_max"
}