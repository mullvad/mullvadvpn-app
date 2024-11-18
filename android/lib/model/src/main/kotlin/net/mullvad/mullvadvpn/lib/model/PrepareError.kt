package net.mullvad.mullvadvpn.lib.model

import android.content.Intent

sealed interface PrepareResult

sealed interface PrepareError : PrepareResult {
    // Result from VpnService.prepare() being invoked with legacy VPN app has always-on
    data object OtherLegacyAlwaysOnVpn : PrepareError

    // Prepare gives intent but there is other always VPN app
    data class OtherAlwaysOnApp(val appName: String) : PrepareError

    data class NotPrepared(val prepareIntent: Intent) : PrepareError
}

data object Prepared : PrepareResult
