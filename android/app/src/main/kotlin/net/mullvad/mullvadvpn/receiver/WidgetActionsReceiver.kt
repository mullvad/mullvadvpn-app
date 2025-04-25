package net.mullvad.mullvadvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS

class WidgetActionsReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        context.startForegroundService(intent.setClassName(context.packageName, VPN_SERVICE_CLASS))
    }
}
