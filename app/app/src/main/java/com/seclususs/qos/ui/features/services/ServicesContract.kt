package com.seclususs.qos.ui.features.services

enum class DaemonStatus {
    ACTIVE, INACTIVE, STARTING, STOPPING, RESTARTING, MISSING
}

data class ServicesState(
    val status: DaemonStatus = DaemonStatus.INACTIVE,
    val cpuUsage: String = "0%",
    val ramUsage: String = "0 MB",
    val uptime: String = "00:00:00",
    val pid: String = "-",
    val cpuProgress: Float = 0f,
    val ramProgress: Float = 0f
)

sealed interface ServicesEvent {
    data object OnStartClicked : ServicesEvent
    data object OnStopClicked : ServicesEvent
    data object OnRestartClicked : ServicesEvent
    data object RefreshMetrics : ServicesEvent
}