package net.mullvad.mullvadvpn.service.notifications

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.os.Build
import android.support.v4.app.NotificationCompat
import net.mullvad.mullvadvpn.R

class NotificationChannel(
    val context: Context,
    val id: String,
    val name: Int,
    val description: Int,
    val importance: Int
) {
    private val badgeColor by lazy {
        context.resources.getColor(R.color.colorPrimary)
    }

    val notificationManager =
        context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    init {
        if (Build.VERSION.SDK_INT >= 26) {
            val channelName = context.getString(name)
            val channelDescription = context.getString(description)

            val channel = NotificationChannel(id, channelName, importance).apply {
                description = channelDescription
                setShowBadge(true)
            }

            notificationManager.createNotificationChannel(channel)
        }
    }

    fun buildNotification(intent: PendingIntent, title: Int): Notification {
        return buildNotification(intent, title, emptyList())
    }

    fun buildNotification(
        pendingIntent: PendingIntent,
        title: Int,
        actions: List<NotificationCompat.Action>
    ): Notification {
        val builder = NotificationCompat.Builder(context, id)
            .setSmallIcon(R.drawable.small_logo_black)
            .setColor(badgeColor)
            .setContentTitle(context.getString(title))
            .setContentIntent(pendingIntent)

        for (action in actions) {
            builder.addAction(action)
        }

        return builder.build()
    }
}
