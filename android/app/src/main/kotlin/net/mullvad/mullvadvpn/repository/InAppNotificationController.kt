package net.mullvad.mullvadvpn.repository

import java.util.UUID
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
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

    data object ShowTunnelStateBlockedNotification : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1000
    }

    data class ShowTunnelStateErrorNotification(val error: ErrorState) : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1001
    }

    data class UnsupportedVersion(val versionInfo: VersionInfo) : InAppNotification() {
        override val statusLevel = StatusLevel.Error
        override val priority: Long = 1001
    }

    data class UpdateAvailable(val versionInfo: VersionInfo) : InAppNotification() {
        override val statusLevel = StatusLevel.Warning
        override val priority: Long = 1001
    }

    data class AccountExpiryNotification(val expiry: DateTime) : InAppNotification() {
        override val statusLevel = StatusLevel.Warning
        override val priority: Long = 1001
    }

    data class NewDeviceNotification(val deviceName: String, val dismiss: () -> Unit) :
        InAppNotification() {
        override val statusLevel = StatusLevel.Info
        override val priority: Long = 1001
    }
}

class InAppNotificationController(
    tunnelStateNotificationsUseCase: TunnelStateNotificationUseCase,
    versionInfoNotificationsUseCase: VersionInfoNotificationUseCase,
    accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase,
    newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    scope: CoroutineScope,
) {
    val notifications =
        combine(
                tunnelStateNotificationsUseCase.notification().map(::listOfNotNull),
                versionInfoNotificationsUseCase.notification(),
                accountExpiryNotificationUseCase.notification().map(::listOfNotNull),
                newDeviceNotificationUseCase.notification().map(::listOfNotNull),
            ) { a, b, c, d ->
                a + b + c + d
            }
            .map {
                it.sortedWith(
                    compareBy(
                        { notification -> notification.statusLevel.ordinal },
                        { notification -> notification.priority }
                    )
                )
            }
            .shareIn(scope, SharingStarted.Eagerly, 1)
}

class AccountExpiryNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
) {
    // TODO this piece of logic should be removed
    private val readyContainer =
        serviceConnectionManager.connectionState.map {
            if (it is ServiceConnectionState.ConnectedReady) {
                it.container
            } else {
                null
            }
        }

    fun notification(): Flow<InAppNotification?> =
        readyContainer.flatMapLatest { container ->
            // TODO This should be done in a neater way
            if (container == null) {
                flowOf(null)
            } else {
                container.accountDataSource.accountExpiry.map { accountExpiry ->
                    accountExpiryNotification(accountExpiry)
                }
            }
        }

    private fun accountExpiryNotification(accountExpiry: AccountExpiry) =
        if (accountExpiry.isCloseToExpiring()) {
            InAppNotification.AccountExpiryNotification(accountExpiry.date() ?: DateTime.now())
        } else null

    private fun AccountExpiry.isCloseToExpiring(): Boolean {
        val threeDaysFromNow = DateTime.now().plusDays(3)
        return this.date()?.isBefore(threeDaysFromNow) == true
    }
}

class TunnelStateNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
) {
    // TODO this piece of logic should be removed
    private val readyContainer =
        serviceConnectionManager.connectionState.map {
            if (it is ServiceConnectionState.ConnectedReady) {
                it.container
            } else {
                null
            }
        }

    fun notification(): Flow<InAppNotification?> =
        readyContainer.flatMapLatest { container ->
            // TODO This should be done in a neater way
            if (container == null) {
                flowOf(null)
            } else {
                container.connectionProxy.tunnelUiStateFlow().distinctUntilChanged().map {
                    uiTunnelState ->
                    tunnelStateNotification(uiTunnelState)
                }
            }
        }

    private fun tunnelStateNotification(tunnelUiState: TunnelState): InAppNotification? =
        when (tunnelUiState) {
            is TunnelState.Connecting -> InAppNotification.ShowTunnelStateBlockedNotification
            is TunnelState.Disconnecting -> {
                if (
                    tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Block ||
                        tunnelUiState.actionAfterDisconnect == ActionAfterDisconnect.Reconnect
                ) {
                    InAppNotification.ShowTunnelStateBlockedNotification
                } else null
            }
            is TunnelState.Error ->
                InAppNotification.ShowTunnelStateErrorNotification(tunnelUiState.errorState)
            else -> null
        }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)
}

class VersionInfoNotificationUseCase(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val isVersionInfoNotificationEnabled: Boolean,
) {
    // TODO This piece of logic should be removed
    private val readyContainer =
        serviceConnectionManager.connectionState.map {
            if (it is ServiceConnectionState.ConnectedReady) {
                it.container
            } else {
                null
            }
        }

    fun notification(): Flow<List<InAppNotification>> =
        readyContainer.flatMapLatest { container ->
            // TODO This should be done in a neater way
            if (container == null) {
                flowOf(emptyList())
            } else {
                container.appVersionInfoCache.appVersionCallbackFlow().map { versionInfo ->
                    listOfNotNull(
                        unsupportedVersionNotification(versionInfo),
                        updateAvailableNotification(versionInfo)
                    )
                }
            }
        }

    private fun updateAvailableNotification(versionInfo: VersionInfo): InAppNotification? {
        if (!isVersionInfoNotificationEnabled) {
            return null
        }

        return if (versionInfo.isOutdated) {
            InAppNotification.UpdateAvailable(versionInfo)
        } else null
    }

    private fun unsupportedVersionNotification(versionInfo: VersionInfo): InAppNotification? {
        if (!isVersionInfoNotificationEnabled) {
            return null
        }

        return if (!versionInfo.isSupported) {
            InAppNotification.UnsupportedVersion(versionInfo)
        } else null
    }
}

class NewDeviceNotificationUseCase(private val deviceRepository: DeviceRepository) {
    private val _mutableShowNewDeviceNotification = MutableStateFlow(false)

    fun notification(): Flow<InAppNotification?> =
        combine(
            deviceRepository.deviceState.map { it.deviceName() },
            _mutableShowNewDeviceNotification
        ) { deviceName, newDeviceCreated ->
            if (newDeviceCreated && deviceName != null) {
                InAppNotification.NewDeviceNotification(
                    deviceName,
                    ::clearNewDeviceCreatedNotification
                )
            } else null
        }

    fun newDeviceCreated() {
        _mutableShowNewDeviceNotification.value = true
    }

    fun clearNewDeviceCreatedNotification() {
        _mutableShowNewDeviceNotification.value = false
    }
}
