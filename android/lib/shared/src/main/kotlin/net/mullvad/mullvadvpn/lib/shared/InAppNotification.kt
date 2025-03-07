package net.mullvad.mullvadvpn.lib.shared

import java.time.Duration
import net.mullvad.mullvadvpn.lib.model.ErrorState

enum class StatusLevel {
    Error,
    Warning,
    Info,
}

sealed class InAppNotification {
    abstract val statusLevel: StatusLevel
    abstract val priority: Long

    data class TunnelStateError(val error: ErrorState) : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1001
    }

    data object TunnelStateBlocked : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1000
    }

    data class UnsupportedVersion(val versionInfo: VersionInfo) : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 999
    }

    data class AccountExpiry(val expiry: Duration) : InAppNotification() {
        override val statusLevel = StatusLevel.Warning
        override val priority: Long = 1001
    }

    data class NewDevice(val deviceName: String) : InAppNotification() {
        override val statusLevel = StatusLevel.Info
        override val priority: Long = 1001
    }

    data object NewVersionChangelog : InAppNotification() {
        override val statusLevel = StatusLevel.Info
        override val priority: Long = 1001
    }
}
