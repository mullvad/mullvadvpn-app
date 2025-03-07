package net.mullvad.mullvadvpn.lib.model

import android.content.Intent

sealed interface PrepareResult

sealed interface PrepareError : PrepareResult {
    // Legacy VPN profile is active as Always-on
    data object OtherLegacyAlwaysOnVpn : PrepareError

    // Another VPN app is active as Always-on (Only works up to Android 11 or debug builds)
    data class OtherAlwaysOnApp(val appName: String) : PrepareError

    // VPN profile can be created or Always-on VPN is active but not detected
    data class NotPrepared(val prepareIntent: Intent) : PrepareError
}

data object Prepared : PrepareResult
