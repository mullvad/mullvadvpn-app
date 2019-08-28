package net.mullvad.mullvadvpn

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.support.v4.app.NotificationCompat

val FOREGROUND_NOTIFICATION_ID: Int = 1

class ForegroundNotificationManager(val service: Service) {
    fun onCreate() {
        service.startForeground(FOREGROUND_NOTIFICATION_ID, buildNotification())
    }

    fun onDestroy() {
        service.stopForeground(FOREGROUND_NOTIFICATION_ID)
    }

    private fun buildNotification(): Notification {
        val intent = Intent(service, MainActivity::class.java)
            .setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .setAction(Intent.ACTION_MAIN)

        val pendingIntent =
            PendingIntent.getActivity(service, 1, intent, PendingIntent.FLAG_UPDATE_CURRENT)

        return NotificationCompat.Builder(service)
            .setSmallIcon(R.drawable.notification)
            .setContentIntent(pendingIntent)
            .build()
    }
}
