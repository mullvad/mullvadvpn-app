package net.mullvad.mullvadvpn.service.notifications

import android.app.Notification as AndroidNotification
import android.app.NotificationManager
import android.app.Service
import android.content.pm.ServiceInfo
import android.net.VpnService
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.R
import net.mullvad.mullvadvpn.model.ChannelId
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.model.NotificationTunnelState
import net.mullvad.mullvadvpn.service.MullvadVpnService

class NotificationHandler(
    private val vpnService: MullvadVpnService,
    private val notificationManager: NotificationManagerCompat,
    private val tunnelStateNotificationUseCase: TunnelStateNotificationUseCase,
    private val scope: CoroutineScope
) {
    private val notificationId: Int = 1

    private val currentNotification =
        tunnelStateNotificationUseCase.notificationState
            .filterNotNull()
            .stateIn(
                scope,
                SharingStarted.Eagerly,
                Notification.Tunnel(
                    state = NotificationTunnelState.Disconnected,
                    actions = emptyList(),
                    ongoing = false
                )
            )

    suspend fun start(foregroundProvider: ShouldBeOnForegroundProvider) {
        val tunnelChannel = ChannelId("vpn_tunnel_status")
        // Todo ensure channel exist
        val channel =
            NotificationChannelCompat.Builder(
                    tunnelChannel.value,
                    NotificationManager.IMPORTANCE_LOW,
                )
                .setName(vpnService.getString(R.string.foreground_notification_channel_name))
                .setDescription(
                    vpnService.getString(R.string.foreground_notification_channel_description)
                )
                .setShowBadge(false)
                .setVibrationEnabled(false)
                .build()
        channel.lockscreenVisibility
        notificationManager.createNotificationChannel(channel)

        //                channel.apply{ lockscreenVisibility =
        // AndroidNotification.VISIBILITY_SECRET }

        scope.launch {
            foregroundProvider.shouldBeOnForeground.collect {
                Log.d("NotificationHandler", "shouldBeOnForeground: $it")
                if (it) {
                    Log.d("NotificationHandler", "Posting foreground notification ")
                    notifyForeground(
                        currentNotification.value.toNotification(
                            context = vpnService,
                            tunnelChannel
                        )
                    )
                } else {
                    vpnService.stopForeground(Service.STOP_FOREGROUND_DETACH)
                }
            }
        }
        scope.launch {
            currentNotification.collect {
                val notification = it.toNotification(context = vpnService, tunnelChannel)

                if (!notificationManager.areNotificationsEnabled()) {
                    return@collect
                }
                if (it.state is NotificationTunnelState.Connected) {
                    Log.d("NotificationHandler", "Connected! Posting it!")
                    notificationManager.notify(notificationId, notification)
                } else if (
                    notificationManager.activeNotifications.any { it.id == notificationId }
                ) {
                    Log.d("NotificationHandler", "Already have a notification active! Posting it!")
                    notificationManager.notify(notificationId, notification)
                }
            }
        }
    }

    private fun notifyForeground(tunnelStateNotification: AndroidNotification) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            if (VpnService.prepare(vpnService) == null) {
                vpnService.startForeground(
                    notificationId,
                    tunnelStateNotification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
                )
            } else {
                // TODO Open app?
                Log.d("NotificationHandler", "VPN permission not granted")
                return
            }
        } else {
            vpnService.startForeground(
                notificationId,
                tunnelStateNotification,
            )
        }
    }
}
