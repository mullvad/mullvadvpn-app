package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider

class AccountExpiryNotificationProvider(channelId: ChannelId) : NotificationProvider {
    override val notifications: Flow<Notification> = emptyFlow()
}
