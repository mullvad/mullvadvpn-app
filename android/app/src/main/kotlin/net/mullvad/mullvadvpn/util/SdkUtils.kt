package net.mullvad.mullvadvpn.util

import android.Manifest
import android.app.PendingIntent
import android.content.Context
import android.content.pm.PackageManager
import android.net.VpnService
import android.os.Build
import android.provider.Settings
import android.service.quicksettings.Tile
import android.util.Log

object SdkUtils {
    fun getSupportedPendingIntentFlags(): Int {
        return if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }
    }

    fun Context.isNotificationPermissionGranted(): Boolean {
        return (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) ||
            checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) ==
                PackageManager.PERMISSION_GRANTED
    }

    fun Context.getAlwaysOnVpnAppName(): String? {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S_V2 || isAccessToHiddenSettingAllowed()) {
            try {
                Settings.Secure.getString(
                    contentResolver,
                    "always_on_vpn_app"
                )?.let { currentAlwaysOnVpn ->
                    var appName = packageManager.getInstalledPackages(0)
                        .filter { it.packageName == currentAlwaysOnVpn }
                    if (appName.size == 1 && appName[0].packageName != packageName) {
                        return appName[0].applicationInfo.loadLabel(packageManager).toString()
                    }
                }
            } catch (ex: SecurityException) {
                Log.e("mullvad", ex.toString())
            }
        }
        return null
    }

    fun Context.isAccessToHiddenSettingAllowed(): Boolean {
        return if ((Build.VERSION.SDK_INT < Build.VERSION_CODES.S_V2)) true else
            try {
                Settings.Secure.getString(
                    contentResolver,
                    "always_on_vpn_app"
                )
                true
            } catch (ex: SecurityException) {
                false
            }
    }

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
