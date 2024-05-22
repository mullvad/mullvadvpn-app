package net.mullvad.mullvadvpn.service.notifications.tunnelstate

import android.util.Log
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
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.permission.VpnPermissionRepository
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.ErrorStateCause
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.model.NotificationAction
import net.mullvad.mullvadvpn.model.NotificationTunnelState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider

class TunnelStateNotificationProvider(
    managementService: ManagementService,
    vpnPermissionRepository: VpnPermissionRepository,
    channelId: ChannelId,
    scope: CoroutineScope
) : NotificationProvider {
    override val notifications: StateFlow<Notification.Tunnel> =
        combine(
                managementService.tunnelState,
                managementService.tunnelState.actionAfterDisconnect().distinctUntilChanged(),
            ) { tunnelState: TunnelState, actionAfterDisconnect: ActionAfterDisconnect?,
                ->
                val notificationTunnelState =
                    tunnelState(
                        tunnelState,
                        actionAfterDisconnect,
                        vpnPermissionRepository.hasVpnPermission(),
                        vpnPermissionRepository.getAlwaysOnVpnAppName()
                    )
                Log.d(
                    "TunnelStateNotificationUseCase",
                    "notificationTunnelState: $notificationTunnelState"
                )
                return@combine Notification.Tunnel(
                    channelId = channelId,
                    state = notificationTunnelState,
                    actions = listOfNotNull(notificationTunnelState.toAction()),
                    ongoing = notificationTunnelState is NotificationTunnelState.Connected
                )
            }
            .stateIn(
                scope,
                SharingStarted.Eagerly,
                Notification.Tunnel(
                    channelId,
                    NotificationTunnelState.Disconnected(true),
                    emptyList(),
                    false
                )
            )

    private fun tunnelState(
        tunnelState: TunnelState,
        actionAfterDisconnect: ActionAfterDisconnect?,
        hasVpnPermission: Boolean,
        alwaysOnVpnPermissionName: String?
    ): NotificationTunnelState =
        tunnelState.toNotificationTunnelState(
            actionAfterDisconnect,
            hasVpnPermission,
            alwaysOnVpnPermissionName
        )

    private fun Flow<TunnelState>.actionAfterDisconnect(): Flow<ActionAfterDisconnect?> =
        filterIsInstance<TunnelState.Disconnecting>()
            .map<TunnelState.Disconnecting, ActionAfterDisconnect?> { it.actionAfterDisconnect }
            .onStart { emit(null) }

    private fun TunnelState.toNotificationTunnelState(
        actionAfterDisconnect: ActionAfterDisconnect?,
        hasVpnPermission: Boolean,
        alwaysOnVpnPermissionName: String?
    ) =
        when (this) {
            is TunnelState.Disconnected -> NotificationTunnelState.Disconnected(hasVpnPermission)
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
            is TunnelState.Error -> toNotificationTunnelState(alwaysOnVpnPermissionName)
        }

    private fun TunnelState.Error.toNotificationTunnelState(
        alwaysOnVpnPermissionName: String?
    ): NotificationTunnelState.Error {
        val cause = errorState.cause
        return when {
            cause is ErrorStateCause.IsOffline -> NotificationTunnelState.Error.DeviceOffline
            cause is ErrorStateCause.InvalidDnsServers -> NotificationTunnelState.Error.Blocking
            cause is ErrorStateCause.VpnPermissionDenied ->
                alwaysOnVpnPermissionName?.let { NotificationTunnelState.Error.AlwaysOnVpn }
                    ?: NotificationTunnelState.Error.VpnPermissionDenied
            errorState.isBlocking -> NotificationTunnelState.Error.Blocking
            else -> NotificationTunnelState.Error.Critical
        }
    }

    private fun NotificationTunnelState.toAction(): NotificationAction.Tunnel =
        when (this) {
            is NotificationTunnelState.Disconnected -> {
                if (this.hasVpnPermission) {
                    NotificationAction.Tunnel.Connect
                } else {
                    NotificationAction.Tunnel.RequestPermission
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
            NotificationTunnelState.Error.AlwaysOnVpn -> NotificationAction.Tunnel.Dismiss
        }
}
