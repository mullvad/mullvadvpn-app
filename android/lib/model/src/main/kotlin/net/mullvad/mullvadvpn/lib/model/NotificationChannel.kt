package net.mullvad.mullvadvpn.lib.model

sealed interface NotificationChannel {
    val id: NotificationChannelId

    data object TunnelUpdates : NotificationChannel {
        private const val CHANNEL_ID = "tunnel_state_notification"
        override val id: NotificationChannelId = NotificationChannelId(CHANNEL_ID)
    }

    data object AccountUpdates : NotificationChannel {
        private const val CHANNEL_ID = "account_updates"
        override val id: NotificationChannelId = NotificationChannelId(CHANNEL_ID)
    }
}
