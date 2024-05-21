package net.mullvad.mullvadvpn.service.notifications

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.toNotification
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.toNotification

class NotificationManager(
    private val notificationManagerCompat: NotificationManagerCompat,
    notificationProviders: List<NotificationProvider>,
    context: Context,
    val scope: CoroutineScope,
) {

    init {

        Log.d("NotificationManager", "Starting NotificationManager with $notificationProviders")

        scope.launch {
            notificationProviders
                .map { it.notifications }
                .merge()
                .collect { notification ->
                    Log.d("NotificationManager", "Posting notification $notification")
                    val androidNotification = notification.toAndroidNotification(context)
                    if (
                        ActivityCompat.checkSelfPermission(
                            context,
                            Manifest.permission.POST_NOTIFICATIONS
                        ) == PackageManager.PERMISSION_GRANTED
                    ) {
                        notificationManagerCompat.notify(notification.id.value, androidNotification)
                    }
                }
        }
    }

    private fun Notification.toAndroidNotification(context: Context): android.app.Notification =
        when (this) {
            is Notification.Tunnel -> toNotification(context)
            is Notification.AccountExpiry -> toNotification(context)
        }
}
