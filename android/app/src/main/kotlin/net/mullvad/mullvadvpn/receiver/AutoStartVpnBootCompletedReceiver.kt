package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import org.koin.core.component.KoinComponent

class AutoStartVpnBootCompletedReceiver : BroadcastReceiver(), KoinComponent {
    override fun onReceive(context: Context?, intent: Intent?) {
        val action = intent?.action
        if (action == Intent.ACTION_BOOT_COMPLETED || action == Intent.ACTION_LOCKED_BOOT_COMPLETED) {
            context?.let { startAndConnectTunnel(context) }
        }
    }

    private fun startAndConnectTunnel(context: Context) {
        val hasVpnPermission = context.prepareVpnSafe().isRight()
        Logger.i("AutoStart on boot and connect, hasVpnPermission: $hasVpnPermission")
        if (hasVpnPermission) {
            val intent =
                Intent().apply {
                    setClassName(context.packageName, VPN_SERVICE_CLASS)
                    action = KEY_CONNECT_ACTION
                }
            context.startForegroundService(intent)
        }
    }
}
