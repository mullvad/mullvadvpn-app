package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun createAccountUri(accountUri: String, websiteAuthToken: WebsiteAuthToken?): Uri {
    val urlString = buildString {
        append(accountUri)
        if (websiteAuthToken != null) {
            append("?token=")
            append(websiteAuthToken.value)
        }
    }
    return Uri.parse(urlString)
}

// NOTE: This function will return the current Always-on VPN package's name. In case of either
// Always-on VPN being disabled or not being able to read the state, NULL will be returned.
fun Context.resolveAlwaysOnVpnPackageName(): String? {
    return try {
        Settings.Secure.getString(contentResolver, ALWAYS_ON_VPN_APP)
    } catch (ex: SecurityException) {
        null
    }
}

fun Context.openVpnSettings() {
    val intent = Intent("android.settings.VPN_SETTINGS")
    startActivity(intent)
}
