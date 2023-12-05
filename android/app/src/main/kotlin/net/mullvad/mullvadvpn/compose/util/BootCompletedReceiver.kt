package net.mullvad.mullvadvpn.compose.util

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

private const val TAG = "AAAAAAAAAAAABootBroadCast"

class BootCompletedReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context?, mBootIntent: Intent?) {
        Log.i(TAG, "BootBroadCast BroadcastReceiver +++STARTED+++")
        Log.d(TAG, "@Boot actionCaught :" + mBootIntent?.action)
        if ("android.intent.action.BOOT_COMPLETED" == mBootIntent?.action) {
            Log.d(TAG, "@Boot actionCaught :" + mBootIntent.action)
            // Now you are getting your Boot receiver
        }
        Log.i(TAG, "BootBroadCast BroadcastReceiver ---END/FIN---")
    }
}
