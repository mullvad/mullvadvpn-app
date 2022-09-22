package net.mullvad.mullvadvpn.util

import android.net.VpnService
import android.os.Build
import android.service.quicksettings.Tile

object SdkUtils {
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
