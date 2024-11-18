package net.mullvad.mullvadvpn.service.notifications.tunnelstate

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationAction
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationId
import net.mullvad.mullvadvpn.lib.model.NotificationTunnelState
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.VpnProfileUseCase
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider

class TunnelStateNotificationProvider(
    connectionProxy: ConnectionProxy,
    vpnPermissionRepository: VpnProfileUseCase,
    deviceRepository: DeviceRepository,
    channelId: NotificationChannelId,
    scope: CoroutineScope,
) : NotificationProvider<Notification.Tunnel> {
    internal val notificationId = NotificationId(2)

    override val notifications: StateFlow<NotificationUpdate<Notification.Tunnel>> =
        combine(
                connectionProxy.tunnelState,
                connectionProxy.tunnelState.actionAfterDisconnect().distinctUntilChanged(),
                deviceRepository.deviceState,
            ) { tunnelState, actionAfterDisconnect, deviceState ->
                if (
                    deviceState is DeviceState.LoggedOut && tunnelState is TunnelState.Disconnected
                ) {
                    return@combine NotificationUpdate.Cancel(notificationId)
                }
                val notificationTunnelState =
                    tunnelState(
                        tunnelState,
                        actionAfterDisconnect,
                        vpnPermissionRepository.prepareVpn().leftOrNull(),
                    )

                return@combine NotificationUpdate.Notify(
                    notificationId,
                    Notification.Tunnel(
                        channelId = channelId,
                        state = notificationTunnelState,
                        actions = listOfNotNull(notificationTunnelState.toAction()),
                        ongoing = notificationTunnelState is NotificationTunnelState.Connected,
                    ),
                )
            }
            .stateIn(scope, SharingStarted.Eagerly, NotificationUpdate.Cancel(notificationId))

    private fun tunnelState(
        tunnelState: TunnelState,
        actionAfterDisconnect: ActionAfterDisconnect?,
        prepareError: PrepareError?,
    ): NotificationTunnelState =
        tunnelState.toNotificationTunnelState(actionAfterDisconnect, prepareError)

    private fun Flow<TunnelState>.actionAfterDisconnect(): Flow<ActionAfterDisconnect?> =
        filterIsInstance<TunnelState.Disconnecting>()
            .map<TunnelState.Disconnecting, ActionAfterDisconnect?> { it.actionAfterDisconnect }
            .onStart { emit(null) }

    private fun TunnelState.toNotificationTunnelState(
        actionAfterDisconnect: ActionAfterDisconnect?,
        prepareError: PrepareError?,
    ) =
        when (this) {
            is TunnelState.Disconnected -> NotificationTunnelState.Disconnected(prepareError)
            is TunnelState.Connecting -> {
                if (actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                    NotificationTunnelState.Reconnecting
                } else {
                    NotificationTunnelState.Connecting
                }
            }
            is TunnelState.Disconnecting -> {
                if (actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                    NotificationTunnelState.Reconnecting
                } else {
                    NotificationTunnelState.Disconnecting
                }
            }
            is TunnelState.Connected -> NotificationTunnelState.Connected
            is TunnelState.Error -> toNotificationTunnelState()
        }

    private fun TunnelState.Error.toNotificationTunnelState(): NotificationTunnelState.Error {
        val cause = errorState.cause
        return when {
            cause is ErrorStateCause.IsOffline && errorState.isBlocking ->
                NotificationTunnelState.Error.DeviceOffline
            cause is ErrorStateCause.InvalidDnsServers -> NotificationTunnelState.Error.Blocking
            cause is ErrorStateCause.OtherLegacyAlwaysOnApp ->
                NotificationTunnelState.Error.LegacyLockdown
            cause is ErrorStateCause.NotPrepared ->
                NotificationTunnelState.Error.VpnPermissionDenied
            cause is ErrorStateCause.OtherAlwaysOnApp ->
                NotificationTunnelState.Error.AlwaysOnVpn(cause.appName)
            errorState.isBlocking -> NotificationTunnelState.Error.Blocking
            else -> NotificationTunnelState.Error.Critical
        }
    }

    private fun NotificationTunnelState.toAction(): NotificationAction.Tunnel =
        when (this) {
            is NotificationTunnelState.Disconnected -> {
                when (prepareError) {
                    is PrepareError.OtherAlwaysOnApp,
                    is PrepareError.OtherLegacyAlwaysOnVpn,
                    null -> NotificationAction.Tunnel.Connect
                    is PrepareError.NotPrepared -> NotificationAction.Tunnel.RequestVpnProfile
                }
            }
            NotificationTunnelState.Disconnecting -> NotificationAction.Tunnel.Connect
            NotificationTunnelState.Connected,
            NotificationTunnelState.Error.Blocking -> NotificationAction.Tunnel.Disconnect
            NotificationTunnelState.Connecting -> NotificationAction.Tunnel.Cancel
            NotificationTunnelState.Reconnecting -> NotificationAction.Tunnel.Cancel
            is NotificationTunnelState.Error.Critical,
            NotificationTunnelState.Error.DeviceOffline,
            NotificationTunnelState.Error.VpnPermissionDenied,
            is NotificationTunnelState.Error.AlwaysOnVpn,
            NotificationTunnelState.Error.LegacyLockdown -> NotificationAction.Tunnel.Dismiss
        }
}
