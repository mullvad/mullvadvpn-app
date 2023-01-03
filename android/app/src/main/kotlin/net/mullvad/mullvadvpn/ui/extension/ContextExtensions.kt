package net.mullvad.mullvadvpn.ui.extension

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.SdkUtils.getInstalledPackagesList

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
    return resolveAlwaysOnVpnPackageName()
        ?.let { currentAlwaysOnVpn ->
            packageManager.getInstalledPackagesList(0)
                .singleOrNull {
                    it.packageName == currentAlwaysOnVpn && it.packageName != packageName
                }
        }?.applicationInfo?.loadLabel(packageManager)?.toString()
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

// NOTE: This function will return the current Always-on VPN package's name. In case of either
// Always-on VPN being disabled or not being able to read the state, NULL will be returned.
fun Context.resolveAlwaysOnVpnPackageName(): String? {
    return try {
        Settings.Secure.getString(
            contentResolver,
            ALWAYS_ON_VPN_APP
        )
    } catch (ex: SecurityException) {
        null
    }
}
