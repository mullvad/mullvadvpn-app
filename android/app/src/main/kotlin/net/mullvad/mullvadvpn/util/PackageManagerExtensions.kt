package net.mullvad.mullvadvpn.util

import android.content.pm.PackageManager
import android.graphics.Bitmap
import androidx.core.graphics.drawable.toBitmapOrNull

fun PackageManager.getApplicationIconBitmapOrNull(packageName: String): Bitmap? =
    try {
        getApplicationIcon(packageName).toBitmapOrNull()
    } catch (e: PackageManager.NameNotFoundException) {
        // Name not found is thrown if the application is not installed
        null
    } catch (e: IllegalArgumentException) {
        // IllegalArgumentException is thrown if the application has an invalid icon
        null
    }
