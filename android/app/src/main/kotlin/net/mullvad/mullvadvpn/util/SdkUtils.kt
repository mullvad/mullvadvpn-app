package net.mullvad.mullvadvpn.util

import android.app.PendingIntent
import android.net.VpnService
import android.os.Build
import android.service.quicksettings.Tile

object SdkUtils {
    fun getSupportedPendingIntentFlags(): Int {
        return if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }
    }

    fun VpnService.Builder.setMeteredIfSupported(isMetered: Boolean) {
        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.Q) {
            this.setMetered(isMetered)
        }
    }

    fun Tile.setSubtitleIfSupported(subtitleText: CharSequence) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            this.subtitle = subtitleText
        }
    }
}
