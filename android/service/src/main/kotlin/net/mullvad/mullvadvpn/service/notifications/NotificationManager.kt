package net.mullvad.mullvadvpn.service.notifications

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.model.NotificationUpdate
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.toNotification
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.toNotification

class NotificationManager(
    private val notificationManagerCompat: NotificationManagerCompat,
    notificationProviders: List<NotificationProvider<Notification>>,
    context: Context,
    val scope: CoroutineScope,
) {

    init {
        scope.launch {
            notificationProviders
                .map { it.notifications }
                .merge()
                .collect { notificationUpdate ->
                    when (notificationUpdate) {
                        is NotificationUpdate.Cancel ->
                            notificationManagerCompat.cancel(
                                notificationUpdate.notificationId.value
                            )
                        is NotificationUpdate.Notify -> {
                            val notification = notificationUpdate.value
                            val androidNotification = notification.toAndroidNotification(context)
                            if (
                                ActivityCompat.checkSelfPermission(
                                    context,
                                    Manifest.permission.POST_NOTIFICATIONS
                                ) == PackageManager.PERMISSION_GRANTED
                            ) {
                                notificationManagerCompat.notify(
                                    notificationUpdate.notificationId.value,
                                    androidNotification
                                )
                            }
                        }
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
