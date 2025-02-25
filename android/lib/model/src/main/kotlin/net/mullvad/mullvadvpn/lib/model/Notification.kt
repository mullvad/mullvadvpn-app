package net.mullvad.mullvadvpn.lib.model

import java.time.Duration

sealed interface Notification {
    val actions: List<NotificationAction>
    val ongoing: Boolean
    val channelId: NotificationChannelId

    data class Tunnel(
        override val channelId: NotificationChannelId,
        val state: NotificationTunnelState,
        override val actions: List<NotificationAction.Tunnel>,
        override val ongoing: Boolean,
    ) : Notification

    data class AccountExpiry(
        override val channelId: NotificationChannelId,
        override val actions: List<NotificationAction.AccountExpiry>,
        val websiteAuthToken: WebsiteAuthToken?,
        val durationUntilExpiry: Duration,
    ) : Notification {
        override val ongoing: Boolean = false
    }
}
