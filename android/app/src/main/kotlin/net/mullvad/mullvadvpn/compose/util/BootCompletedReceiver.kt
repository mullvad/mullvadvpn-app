package net.mullvad.mullvadvpn.compose.util

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import net.mullvad.mullvadvpn.di.APP_PREFERENCES_NAME
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.repository.IS_CONNECT_ON_BOOT_ENABLED_KEY

private const val TAG = "AAAAAAAAAAAABootBroadCast"

class BootCompletedReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context?, mBootIntent: Intent?) {
        if (mBootIntent?.action == "android.intent.action.BOOT_COMPLETED") {
            context?.let {
                if (isConnectOnBootEnabled(it)) {
                    startDaemonService(it)
                }
            }
        }
    }

    private fun isConnectOnBootEnabled(context: Context): Boolean {
        return context
            .getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
            .getBoolean(IS_CONNECT_ON_BOOT_ENABLED_KEY, false)
    }

    private fun startDaemonService(context: Context) {
        val intent =
            Intent().apply {
                setClassName(context.packageName, VPN_SERVICE_CLASS)
                action = KEY_CONNECT_ACTION
            }
        context.startForegroundService(intent)
    }
}
