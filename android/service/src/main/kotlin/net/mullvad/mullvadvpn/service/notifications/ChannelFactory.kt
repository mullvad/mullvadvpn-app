package net.mullvad.mullvadvpn.service.notifications

import android.app.NotificationManager
import android.content.res.Resources
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationManagerCompat
import net.mullvad.mullvadvpn.lib.common.R
import net.mullvad.mullvadvpn.lib.model.ChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationChannel

class ChannelFactory(
    private val notificationManagerCompat: NotificationManagerCompat,
    private val resources: Resources,
    channels: List<NotificationChannel>
) {
    init {
        channels.forEach { createChannel(it) }
    }

    private fun createChannel(channel: NotificationChannel): ChannelId {
        val androidChannel = channel.toAndroidNotificationChannel()
        notificationManagerCompat.createNotificationChannel(androidChannel)
        return channel.id
    }

    private fun NotificationChannel.toAndroidNotificationChannel(): NotificationChannelCompat =
        when (this) {
            NotificationChannel.AccountUpdates -> NotificationChannel.AccountUpdates.toChannel()
            NotificationChannel.TunnelUpdates -> NotificationChannel.TunnelUpdates.toChannel()
        }

    private fun NotificationChannel.TunnelUpdates.toChannel(): NotificationChannelCompat =
        NotificationChannelCompat.Builder(
                id.value,
                NotificationManager.IMPORTANCE_LOW,
            )
            .setName(resources.getString(R.string.foreground_notification_channel_name))
            .setDescription(
                resources.getString(R.string.foreground_notification_channel_description)
            )
            .setShowBadge(false)
            .setVibrationEnabled(false)
            .build()

    private fun NotificationChannel.AccountUpdates.toChannel(): NotificationChannelCompat =
        NotificationChannelCompat.Builder(
                id.value,
                NotificationManager.IMPORTANCE_HIGH,
            )
            .setName(resources.getString(R.string.account_time_notification_channel_name))
            .setDescription(
                resources.getString(R.string.account_time_notification_channel_description)
            )
            .setShowBadge(true)
            .setVibrationEnabled(true)
            .build()
    //    NotificationCompat.VISIBILITY_PRIVATE,
}
