package net.mullvad.talpid.util

import android.net.VpnService
import android.os.Build

object TalpidSdkUtils {
    fun VpnService.Builder.setMeteredIfSupported(isMetered: Boolean) {
        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.Q) {
            this.setMetered(isMetered)
        }
    }
}
