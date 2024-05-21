package net.mullvad.mullvadvpn.model

import org.joda.time.Duration

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
