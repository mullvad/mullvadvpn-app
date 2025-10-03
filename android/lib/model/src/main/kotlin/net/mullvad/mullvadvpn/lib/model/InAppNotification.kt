package net.mullvad.mullvadvpn.lib.model

import java.time.Duration

enum class StatusLevel {
    Error,
    Warning,
    Info,
    None,
}

sealed class InAppNotification {
    abstract val statusLevel: StatusLevel
    abstract val priority: Long

    data class TunnelStateError(val error: ErrorState) : InAppNotification() {
        override val statusLevel =
            if (error.cause is ErrorStateCause.IsOffline) {
                StatusLevel.Warning
            } else {
                StatusLevel.Error
            }
        override val priority: Long = 1005
    }

    data object Android16UpgradeWarning : InAppNotification() {
        override val statusLevel = StatusLevel.Warning
        override val priority: Long = 1005
    }

    data object TunnelStateBlocked : InAppNotification() {
        override val statusLevel = StatusLevel.None
        override val priority: Long = 1004
    }

    data class UnsupportedVersion(val versionInfo: VersionInfo) : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1002
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
        override val priority: Long = 1000
    }
}
