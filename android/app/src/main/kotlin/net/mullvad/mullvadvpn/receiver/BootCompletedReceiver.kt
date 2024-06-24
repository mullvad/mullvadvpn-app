package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.VpnService
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS

class BootCompletedReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == Intent.ACTION_BOOT_COMPLETED) {
            context?.let { startAndConnectTunnel(context) }
        }
    }

    private fun startAndConnectTunnel(context: Context) {
        val hasVpnPermission = VpnService.prepare(context) == null
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
