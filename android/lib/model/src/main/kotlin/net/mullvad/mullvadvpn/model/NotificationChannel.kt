package net.mullvad.mullvadvpn.model

import java.net.InetAddress

@JvmInline value class ChannelId(val value: String)

sealed interface NotificationChannel {
    val id: ChannelId

    data class TunnelUpdates(override val id: ChannelId) : NotificationChannel
}

sealed interface Notification {
    val actions: List<NotificationAction>
    val ongoing: Boolean
    val channelId: ChannelId

    data class Tunnel(
        val state: NotificationTunnelState,
        override val actions: List<NotificationAction.Tunnel>,
        override val ongoing: Boolean,
    ) : Notification {
        override val channelId: ChannelId = ChannelId("tunnel")
    }
}

sealed interface NotificationAction {

    sealed interface Tunnel : NotificationAction {
        data object Connect : Tunnel

        data object Disconnect : Tunnel

        data object Cancel : Tunnel

        data object Dismiss : Tunnel
    }
}

sealed interface NotificationTunnelState {
    data object Disconnected : NotificationTunnelState

    data object Connecting : NotificationTunnelState

    data object Connected : NotificationTunnelState

    data object Reconnecting : NotificationTunnelState

    data object Disconnecting : NotificationTunnelState

    sealed interface Error : NotificationTunnelState {
        data object DeviceOffline : Error

        data object Blocking : Error

        data object VpnPermissionDenied : Error

        data class InvalidDnsServers(val addresses: List<InetAddress>) : Error

        data class Critical(val cause: ErrorStateCause) : Error
    }
}
