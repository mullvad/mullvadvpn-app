package net.mullvad.mullvadvpn.lib.common.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import net.mullvad.mullvadvpn.lib.common.R
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.getInstalledPackagesList

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun Context.openAccountPageInBrowser(authToken: String) {
    startActivity(
        Intent(Intent.ACTION_VIEW, Uri.parse(getString(R.string.account_url) + "?token=$authToken"))
    )
}

fun Context.getAlwaysOnVpnAppName(): String? {
    return resolveAlwaysOnVpnPackageName()
        ?.let { currentAlwaysOnVpn ->
            packageManager.getInstalledPackagesList(0).singleOrNull {
                it.packageName == currentAlwaysOnVpn && it.packageName != packageName
            }
        }
        ?.applicationInfo
        ?.loadLabel(packageManager)
        ?.toString()
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

fun Context.openLink(uri: Uri) {
    val intent = Intent(Intent.ACTION_VIEW, uri)
    startActivity(intent)
}

fun Context.openVpnSettings() {
    val intent = Intent("android.settings.VPN_SETTINGS")
    startActivity(intent)
}

fun Context.vpnSettingsAvailable(): Boolean =
    Intent("android.net.vpn.SETTINGS").resolveActivity(packageManager) != null
