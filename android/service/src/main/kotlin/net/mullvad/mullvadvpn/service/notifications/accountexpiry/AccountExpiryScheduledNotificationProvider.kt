package net.mullvad.mullvadvpn.service.notifications.accountexpiry

import java.time.Duration
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationId
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.service.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider

class AccountExpiryScheduledNotificationProvider(
    private val channelId: NotificationChannelId,
    private val accountRepository: AccountRepository,
) : NotificationProvider<Notification.AccountExpiry> {

    private val notificationChannel: Channel<NotificationUpdate<Notification.AccountExpiry>> =
        Channel(Channel.CONFLATED)

    override val notifications: Flow<NotificationUpdate<Notification.AccountExpiry>>
        get() = notificationChannel.receiveAsFlow()

    suspend fun showNotification(durationUntilExpiry: Duration) {
        val notification =
            Notification.AccountExpiry(
                channelId = channelId,
                actions = emptyList(),
                websiteAuthToken =
                    if (MullvadVpnService.daemonInitialized.get() && !IS_PLAY_BUILD)
                        accountRepository.getWebsiteAuthToken()
                    else null,
                durationUntilExpiry = durationUntilExpiry,
            )

        val notificationUpdate = NotificationUpdate.Notify(NOTIFICATION_ID, notification)
        notificationChannel.send(notificationUpdate)
    }

    companion object {
        private val NOTIFICATION_ID = NotificationId(3)
    }
}
