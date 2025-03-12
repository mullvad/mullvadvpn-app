package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.net.toUri
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

fun createAccountUri(accountUri: String, websiteAuthToken: WebsiteAuthToken?): Uri {
    val urlString = buildString {
        append(accountUri)
        if (websiteAuthToken != null) {
            append("?token=")
            append(websiteAuthToken.value)
        }
    }
    return urlString.toUri()
}

fun Context.openVpnSettings() {
    val intent = Intent("android.settings.VPN_SETTINGS")
    startActivity(intent)
}
