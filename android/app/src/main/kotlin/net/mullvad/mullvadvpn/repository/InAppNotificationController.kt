package net.mullvad.mullvadvpn.repository

import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.usecase.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import org.joda.time.Duration

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
}

class InAppNotificationController(
    accountExpiryInAppNotificationUseCase: AccountExpiryInAppNotificationUseCase,
    newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    versionNotificationUseCase: VersionNotificationUseCase,
    tunnelStateNotificationUseCase: TunnelStateNotificationUseCase,
    scope: CoroutineScope,
) {

    val notifications =
        combine(
                tunnelStateNotificationUseCase().onEach {
                    Logger.d("tunnelStateNotificationUseCase: $it")
                },
                versionNotificationUseCase().onEach { Logger.d("versionNotificationUseCase: $it") },
                accountExpiryInAppNotificationUseCase().onEach {
                    Logger.d("accountExpiryInAppNotificationUseCase: $it")
                },
                newDeviceNotificationUseCase().onEach {
                    Logger.d("newDeviceNotificationUseCase: $it")
                },
            ) { a, b, c, d ->
                a + b + c + d
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
