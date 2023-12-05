package net.mullvad.mullvadvpn.compose.util

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import net.mullvad.mullvadvpn.di.APP_PREFERENCES_NAME
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS

private const val IS_CONNECT_ON_BOOT_ENABLED_KEY = "is_connect_on_boot_enabled"
private const val TAG = "AAAAAAAAAAAABootBroadCast"

class BootCompletedReceiver : BroadcastReceiver() {

    override fun onReceive(context: Context?, mBootIntent: Intent?) {
        Log.d(TAG, "AAAAA @Boot ")
        if ("android.intent.action.BOOT_COMPLETED" == mBootIntent?.action) {
            Log.d(TAG, "AAAAA @Boot actionCaught :" + mBootIntent.action)
            // Now you are getting your Boot receiver
            context?.let {
                Log.d(TAG, "AAAAA @Boot context :" + mBootIntent.action)
                if (isConnectOnBootEnabled(it)) {
                    Log.d(TAG, "AAAAA @Boot Service :" + mBootIntent.action)
                    toggleTunnel(it)
                } else {

                    Log.d(TAG, "AAAAA @Boot  isConnectOnBootEnabled is false:")
                }
            }
        }
    }

    private fun isConnectOnBootEnabled(context: Context): Boolean {
        return context
            .getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
            .getBoolean(IS_CONNECT_ON_BOOT_ENABLED_KEY, false)
    }

    private fun toggleTunnel(context: Context) {
        val intent =
            Intent().apply {
                setClassName(context.packageName, VPN_SERVICE_CLASS)
                action = KEY_CONNECT_ACTION
            }

        // Always start as foreground in case tile is out-of-sync.
        context.startForegroundService(intent)
    }
}
