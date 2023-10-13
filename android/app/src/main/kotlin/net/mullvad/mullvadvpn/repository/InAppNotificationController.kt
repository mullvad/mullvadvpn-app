package net.mullvad.mullvadvpn.repository

import java.util.UUID
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import net.mullvad.talpid.tunnel.ErrorState
import org.joda.time.DateTime

enum class StatusLevel {
    Error,
    Warning,
    Info,
}

sealed class InAppNotification {
    val uuid: UUID = UUID.randomUUID()
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

    data class AccountExpiry(val expiry: DateTime) : InAppNotification() {
        override val statusLevel = StatusLevel.Warning
        override val priority: Long = 1001
    }

    data class NewDevice(val deviceName: String) : InAppNotification() {
        override val statusLevel = StatusLevel.Info
        override val priority: Long = 1001
    }

    data class UpdateAvailable(val versionInfo: VersionInfo) : InAppNotification() {
        override val statusLevel = StatusLevel.Info
        override val priority: Long = 1000
    }
}

class InAppNotificationController(
    accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase,
    newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    versionNotificationUseCase: VersionNotificationUseCase,
    tunnelStateNotificationUseCase: TunnelStateNotificationUseCase,
    scope: CoroutineScope,
) {

    val notifications =
        combine(
                tunnelStateNotificationUseCase.notifications(),
                versionNotificationUseCase.notifications(),
                accountExpiryNotificationUseCase.notifications(),
                newDeviceNotificationUseCase.notifications(),
            ) { a, b, c, d ->
                a + b + c + d
            }
            .map {
                it.sortedWith(
                    compareBy(
                        { notification -> notification.statusLevel.ordinal },
                        { notification -> -notification.priority }
                    )
                )
            }
            .stateIn(scope, SharingStarted.Eagerly, emptyList())
}
