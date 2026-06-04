package com.seclususs.qos.ui.features.daemon

enum class DaemonStatus {
    SEARCHING, ACTIVE, INACTIVE, STOPPING, MISSING
}

data class DaemonState(
    val status: DaemonStatus = DaemonStatus.SEARCHING,
    val needsReboot: Boolean = false,
    val uptime: String = "-",
    val pid: String = "-"
)

sealed interface DaemonEvent {
    data object OnStopClicked : DaemonEvent
    data object OnRebootClicked : DaemonEvent
    data object RefreshInfo : DaemonEvent
}