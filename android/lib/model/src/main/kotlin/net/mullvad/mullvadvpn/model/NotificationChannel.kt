package net.mullvad.mullvadvpn.model

import org.joda.time.Duration

@JvmInline value class ChannelId(val value: String)

sealed interface NotificationChannel {
    val id: ChannelId

    data object TunnelUpdates : NotificationChannel {
        override val id: ChannelId = ChannelId("tunnel_state_notification")
    }

    data object AccountUpdates : NotificationChannel {
        override val id: ChannelId = ChannelId("account_updates")
    }
}

@JvmInline value class NotificationId(val value: Int)

sealed interface NotificationUpdate<out D> {
    val notificationId: NotificationId

    data class Notify<D>(override val notificationId: NotificationId, val value: D) :
        NotificationUpdate<D>

    data class Cancel(override val notificationId: NotificationId) : NotificationUpdate<Nothing>
}

sealed interface Notification {
    val actions: List<NotificationAction>
    val ongoing: Boolean
    val channelId: ChannelId

    data class Tunnel(
        override val channelId: ChannelId,
        val state: NotificationTunnelState,
        override val actions: List<NotificationAction.Tunnel>,
        override val ongoing: Boolean,
    ) : Notification

    data class AccountExpiry(
        override val channelId: ChannelId,
        override val actions: List<NotificationAction.AccountExpiry>,
        val wwwAuthToken: WwwAuthToken?,
        val isPlayBuild: Boolean,
        val durationUntilExpiry: Duration
    ) : Notification {
        override val ongoing: Boolean = false
    }
}

sealed interface NotificationAction {

    sealed interface AccountExpiry : NotificationAction {
        data object Open : AccountExpiry
    }

    sealed interface Tunnel : NotificationAction {
        data object Connect : Tunnel

        data object Disconnect : Tunnel

        data object Cancel : Tunnel

        data object Dismiss : Tunnel

        data object RequestPermission : Tunnel
    }
}

sealed interface NotificationTunnelState {
    data class Disconnected(val hasVpnPermission: Boolean) : NotificationTunnelState

    data object Connecting : NotificationTunnelState

    data object Connected : NotificationTunnelState

    data object Reconnecting : NotificationTunnelState

    data object Disconnecting : NotificationTunnelState

    sealed interface Error : NotificationTunnelState {
        data object DeviceOffline : Error

        data object Blocking : Error

        data object VpnPermissionDenied : Error

        data object AlwaysOnVpn : Error

        data object Critical : Error
    }
}
