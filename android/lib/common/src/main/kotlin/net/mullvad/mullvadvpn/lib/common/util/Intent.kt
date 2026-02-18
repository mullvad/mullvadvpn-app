package net.mullvad.mullvadvpn.lib.common.util

import android.app.PendingIntent
import android.app.PendingIntent.FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT
import android.os.Build

fun getSupportedPendingIntentFlags(): Int {
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
        PendingIntent.FLAG_UPDATE_CURRENT or
            PendingIntent.FLAG_MUTABLE or
            FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
    } else {
        PendingIntent.FLAG_UPDATE_CURRENT
    }
}
