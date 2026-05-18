package com.seclususs.qos.domain.model

data class DaemonMetrics(
    val cpuUsage: String = "0%", val ramUsage: String = "0 MB", val uptime: String = "00:00:00"
)