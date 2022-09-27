package net.mullvad.mullvadvpn.service.notifications

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import net.mullvad.mullvadvpn.R

class NotificationChannel(
    val context: Context,
    val id: String,
    val visibility: Int,
    name: Int,
    description: Int,
    importance: Int,
    isVibrationEnabled: Boolean,
    isBadgeEnabled: Boolean
) {
    private val badgeColor by lazy {
        context.getColor(R.color.colorPrimary)
    }

    val notificationManager = NotificationManagerCompat.from(context)

    init {
        val channelName = context.getString(name)
        val channelDescription = context.getString(description)

        val channel = NotificationChannelCompat.Builder(id, importance)
            .setName(channelName)
            .setDescription(channelDescription)
            .setShowBadge(isBadgeEnabled)
            .setVibrationEnabled(isVibrationEnabled)
            .build()

        notificationManager.createNotificationChannel(channel)
    }

    fun buildNotification(
        intent: PendingIntent,
        title: String,
        deleteIntent: PendingIntent? = null,
        isOngoing: Boolean = false
    ): Notification {
        return buildNotification(intent, title, emptyList(), deleteIntent, isOngoing)
    }

    fun buildNotification(
        intent: PendingIntent,
        title: Int,
        deleteIntent: PendingIntent? = null,
        isOngoing: Boolean = false
    ): Notification {
        return buildNotification(intent, title, emptyList(), deleteIntent, isOngoing)
    }

    fun buildNotification(
        pendingIntent: PendingIntent,
        title: Int,
        actions: List<NotificationCompat.Action>,
        deleteIntent: PendingIntent? = null,
        isOngoing: Boolean = false
    ): Notification {
        return buildNotification(
            pendingIntent,
            context.getString(title),
            actions,
            deleteIntent,
            isOngoing
        )
    }

    private fun buildNotification(
        pendingIntent: PendingIntent,
        title: String,
        actions: List<NotificationCompat.Action>,
        deleteIntent: PendingIntent? = null,
        isOngoing: Boolean = false
    ): Notification {
        val builder = NotificationCompat.Builder(context, id)
            .setSmallIcon(R.drawable.small_logo_black)
            .setColor(badgeColor)
            .setContentTitle(title)
            .setContentIntent(pendingIntent)
            .setVisibility(visibility)
            .setOngoing(isOngoing)
        for (action in actions) {
            builder.addAction(action)
        }

        deleteIntent?.let { intent ->
            builder.setDeleteIntent(intent)
        }

        return builder.build()
    }
}
