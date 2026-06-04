package com.seclususs.qos.ui.features.services

enum class DaemonStatus {
    ACTIVE, INACTIVE, STOPPING, MISSING
}

data class ServicesState(
    val status: DaemonStatus = DaemonStatus.INACTIVE,
    val needsReboot: Boolean = false,
    val uptime: String = "-",
    val pid: String = "-"
)

sealed interface ServicesEvent {
    data object OnStopClicked : ServicesEvent
    data object OnRebootClicked : ServicesEvent
    data object RefreshInfo : ServicesEvent
}