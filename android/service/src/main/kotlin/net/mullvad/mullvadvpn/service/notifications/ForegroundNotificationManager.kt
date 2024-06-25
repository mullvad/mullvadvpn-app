package net.mullvad.mullvadvpn.service.notifications

import android.app.Service
import android.content.pm.ServiceInfo
import android.net.VpnService
import android.os.Build
import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannel
import net.mullvad.mullvadvpn.lib.model.NotificationTunnelState
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.TunnelStateNotificationProvider
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.toNotification

class ForegroundNotificationManager(
    private val vpnService: MullvadVpnService,
    private val tunnelStateNotificationProvider: TunnelStateNotificationProvider,
    private val scope: CoroutineScope,
) {
    suspend fun start(foregroundProvider: ShouldBeOnForegroundProvider) {
        scope.launch {
            foregroundProvider.shouldBeOnForeground.collect {
                if (it) {
                    Logger.d("Starting foreground")
                    notifyForeground(getTunnelStateNotificationOrDefault())
                } else {
                    Logger.d("Stopping foreground")
                    vpnService.stopForeground(Service.STOP_FOREGROUND_DETACH)
                }
            }
        }
    }

    private fun getTunnelStateNotificationOrDefault(): Notification.Tunnel {
        val current = tunnelStateNotificationProvider.notifications.value

        return if (current is NotificationUpdate.Notify) {
            current.value
        } else {
            defaultNotification
        }
    }

    private fun notifyForeground(tunnelStateNotification: Notification.Tunnel) {

        val androidNotification = tunnelStateNotification.toNotification(vpnService)
        if (VpnService.prepare(vpnService) != null) {
            // Got connect/disconnect intent, but we  don't have permission to go in foreground.
            // tunnel state will return permission and we will eventually get stopped by system.
            Logger.d("Did not start foreground: VPN permission not granted")
            return
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            Logger.d("Starting foreground UPSIDE_DOWN_CAKE")
            vpnService.startForeground(
                tunnelStateNotificationProvider.notificationId.value,
                androidNotification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
            )
        } else {
            vpnService.startForeground(
                tunnelStateNotificationProvider.notificationId.value,
                androidNotification,
            )
        }
    }

    private val defaultNotification =
        Notification.Tunnel(
            NotificationChannel.TunnelUpdates.id,
            NotificationTunnelState.Disconnected(true),
            emptyList(),
            false
        )
}
