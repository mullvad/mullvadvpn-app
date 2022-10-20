package net.mullvad.mullvadvpn.util

import android.Manifest
import android.app.PendingIntent
import android.content.Context
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.net.VpnService
import android.os.Build
import android.service.quicksettings.Tile

object SdkUtils {
    fun getSupportedPendingIntentFlags(): Int {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
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

    fun PackageManager.getInstalledPackagesList(flags: Int = 0): List<PackageInfo> =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            getInstalledPackages(PackageManager.PackageInfoFlags.of(flags.toLong()))
        } else {
            @Suppress("DEPRECATION")
            getInstalledPackages(flags)
        }
}
