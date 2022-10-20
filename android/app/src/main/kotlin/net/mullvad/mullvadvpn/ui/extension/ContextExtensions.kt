package net.mullvad.mullvadvpn.ui.extension

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.MainActivity

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun Context.openAccountPageInBrowser(authToken: String) {
    startActivity(
        Intent(
            Intent.ACTION_VIEW,
            Uri.parse(getString(R.string.account_url) + "?token=$authToken")
        )
    )
}

fun Context.getAlwaysOnVpnAppName(): String? {
    return resolveAlwaysOnVpnPackageName()?.let { currentAlwaysOnVpn ->
        var appName = packageManager.getInstalledPackages(0)
            .filter { it.packageName == currentAlwaysOnVpn }
        if (appName.size == 1 && appName[0].packageName != packageName) {
            appName[0].applicationInfo.loadLabel(packageManager).toString()
        } else {
            null
        }
    } ?: run {
        null
    }
}

fun Fragment.requireMainActivity(): MainActivity {
    return if (this.activity is MainActivity) {
        this.activity as MainActivity
    } else {
        throw IllegalStateException(
            "Fragment $this not attached to ${MainActivity::class.simpleName}."
        )
    }
}

private fun Context.resolveAlwaysOnVpnPackageName(): String? {
    return try {
        Settings.Secure.getString(
            contentResolver,
            ALWAYS_ON_VPN_APP
        )
    } catch (ex: SecurityException) {
        null
    }
}
