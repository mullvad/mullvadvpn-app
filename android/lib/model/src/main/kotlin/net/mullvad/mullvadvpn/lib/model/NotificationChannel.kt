package net.mullvad.mullvadvpn.lib.model

sealed interface NotificationChannel {
    val id: ChannelId

    data object TunnelUpdates : NotificationChannel {
        override val id: ChannelId = ChannelId("tunnel_state_notification")
    }

    data object AccountUpdates : NotificationChannel {
        override val id: ChannelId = ChannelId("account_updates")
    }
}
