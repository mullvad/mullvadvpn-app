package net.mullvad.mullvadvpn.lib.model

sealed interface NotificationChannel {
    val id: NotificationChannelId

    data object TunnelUpdates : NotificationChannel {
        override val id: NotificationChannelId = NotificationChannelId("tunnel_state_notification")
    }

    data object AccountUpdates : NotificationChannel {
        override val id: NotificationChannelId = NotificationChannelId("account_updates")
    }
}
