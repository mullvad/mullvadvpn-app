package net.mullvad.mullvadvpn.repository

import java.time.Duration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.usecase.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase

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

class InAppNotificationController(
    accountExpiryInAppNotificationUseCase: AccountExpiryInAppNotificationUseCase,
    newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    newChangelogNotificationUseCase: NewChangelogNotificationUseCase,
    versionNotificationUseCase: VersionNotificationUseCase,
    tunnelStateNotificationUseCase: TunnelStateNotificationUseCase,
    scope: CoroutineScope,
) {

    val notifications =
        combine(
                tunnelStateNotificationUseCase(),
                versionNotificationUseCase(),
                accountExpiryInAppNotificationUseCase(),
                newDeviceNotificationUseCase(),
                newChangelogNotificationUseCase(),
            ) { a, b, c, d, e ->
                a + b + c + d + e
            }
            .map {
                it.sortedWith(
                    compareBy(
                        { notification -> notification.statusLevel.ordinal },
                        { notification -> -notification.priority },
                    )
                )
            }
            .stateIn(scope, SharingStarted.Eagerly, emptyList())
}
