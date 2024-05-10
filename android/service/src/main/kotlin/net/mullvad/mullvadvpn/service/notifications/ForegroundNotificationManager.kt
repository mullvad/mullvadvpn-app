package net.mullvad.mullvadvpn.service.notifications

import android.app.Service
import android.content.pm.ServiceInfo
import android.net.VpnService
import android.os.Build
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Notification
import net.mullvad.mullvadvpn.service.MullvadVpnService
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.TunnelStateNotificationProvider
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.toNotification

class ForegroundNotificationManager(
    private val vpnService: MullvadVpnService,
    private val tunnelStateNotificationProvider: TunnelStateNotificationProvider,
    private val scope: CoroutineScope,
) {
    suspend fun start(foregroundProvider: ShouldBeOnForegroundProvider) {

        //                channel.apply{ lockscreenVisibility =
        // AndroidNotification.VISIBILITY_SECRET }

        scope.launch {
            foregroundProvider.shouldBeOnForeground.collect {
                Log.d("ForegroundNotificationManager", "shouldBeOnForeground: $it")
                if (it) {
                    Log.d("ForegroundNotificationManager", "Posting foreground notification ")
                    notifyForeground(tunnelStateNotificationProvider.notifications.value)
                } else {
                    vpnService.stopForeground(Service.STOP_FOREGROUND_DETACH)
                }
            }
        }
    }

    private fun notifyForeground(tunnelStateNotification: Notification.Tunnel) {

        val androidNotification = tunnelStateNotification.toNotification(vpnService)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            if (VpnService.prepare(vpnService) == null) {
                vpnService.startForeground(
                    tunnelStateNotification.id.value,
                    androidNotification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
                )
            } else {
                // TODO Open app?
                // Connect intent but no permission
                Log.d("NotificationHandler", "VPN permission not granted")
                return
            }
        } else {
            vpnService.startForeground(
                tunnelStateNotification.id.value,
                androidNotification,
            )
        }
    }
}
