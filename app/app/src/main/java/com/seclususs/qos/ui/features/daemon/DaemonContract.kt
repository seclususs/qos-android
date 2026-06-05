package com.seclususs.qos.ui.features.daemon

import androidx.compose.runtime.Stable

enum class DaemonStatus {
    SEARCHING, ACTIVE, INACTIVE, STOPPING, MISSING
}

@Stable
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