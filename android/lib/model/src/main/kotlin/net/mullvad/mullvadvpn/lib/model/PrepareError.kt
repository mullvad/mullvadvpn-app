package net.mullvad.mullvadvpn.lib.model

import android.content.Intent

sealed interface PrepareResult

sealed interface PrepareError : PrepareResult {
    // Legacy VPN profile is active as Always-on
    data object OtherLegacyAlwaysOnVpn : PrepareError

    // Another VPN app is active as Always-on
    data class OtherAlwaysOnApp(val appName: String) : PrepareError

    data class NotPrepared(val prepareIntent: Intent) : PrepareError
}

data object Prepared : PrepareResult
