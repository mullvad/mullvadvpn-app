package net.mullvad.mullvadvpn.lib.common.util

import android.app.PendingIntent
import android.app.PendingIntent.FLAG_IMMUTABLE
import android.os.Build

fun getSupportedPendingIntentFlags(): Int {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
        PendingIntent.FLAG_UPDATE_CURRENT or FLAG_IMMUTABLE
    } else {
        PendingIntent.FLAG_UPDATE_CURRENT or FLAG_IMMUTABLE
    }
}
