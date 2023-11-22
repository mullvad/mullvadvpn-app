package net.mullvad.mullvadvpn.lib.common.util

import android.Manifest
import android.app.PendingIntent
import android.app.PendingIntent.FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT
import android.content.Context
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.os.Build
import android.service.quicksettings.Tile
import android.widget.Toast

object SdkUtils {
    fun getSupportedPendingIntentFlags(): Int {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            PendingIntent.FLAG_UPDATE_CURRENT or
                PendingIntent.FLAG_MUTABLE or
                FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT
        } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
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

    fun Tile.setSubtitleIfSupported(subtitleText: CharSequence) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            this.subtitle = subtitleText
        }
    }

    fun PackageManager.getInstalledPackagesList(flags: Int = 0): List<PackageInfo> =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            getInstalledPackages(PackageManager.PackageInfoFlags.of(flags.toLong()))
        } else {
            @Suppress("DEPRECATION") getInstalledPackages(flags)
        }

    fun showCopyToastIfNeeded(context: Context, message: String) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
        }
    }
}
