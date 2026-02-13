package net.mullvad.mullvadvpn.lib.pushnotification.accountexpiry

import co.touchlab.kermit.Logger
import java.time.Duration
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationId
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.pushnotification.NotificationProvider

class AccountExpiryNotificationProvider(private val channelId: NotificationChannelId) :
    NotificationProvider<Notification.AccountExpiry> {

    private val notificationChannel: Channel<NotificationUpdate<Notification.AccountExpiry>> =
        Channel(Channel.CONFLATED)

    override val notifications: Flow<NotificationUpdate<Notification.AccountExpiry>>
        get() = notificationChannel.receiveAsFlow()

    fun showNotification(durationUntilExpiry: Duration) {
        val notification =
            Notification.AccountExpiry(
                channelId = channelId,
                actions = emptyList(),
                durationUntilExpiry = durationUntilExpiry,
            )

        val notificationUpdate = NotificationUpdate.Notify(NOTIFICATION_ID, notification)
        // Always succeeds because the channel is conflated.
        notificationChannel.trySend(notificationUpdate)
    }

    suspend fun cancelNotification() {
        Logger.d("Cancelling existing account expiry notification")
        val notificationUpdate = NotificationUpdate.Cancel(NOTIFICATION_ID)
        notificationChannel.send(notificationUpdate)
    }

    companion object {
        private val NOTIFICATION_ID = NotificationId(3)
    }
}
