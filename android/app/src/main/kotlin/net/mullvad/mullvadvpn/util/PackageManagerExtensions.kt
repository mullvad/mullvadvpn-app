package net.mullvad.mullvadvpn.util

import android.content.pm.PackageManager
import android.graphics.Bitmap
import androidx.core.graphics.drawable.toBitmapOrNull

fun PackageManager.getApplicationIconBitmapOrNull(packageName: String): Bitmap? =
    try {
        getApplicationIcon(packageName).toBitmapOrNull()
    } catch (e: Exception) {
        // Name not found is thrown if the application is not installed
        // IllegalArgumentException is thrown if the application has an invalid icon
        when (e) {
            is PackageManager.NameNotFoundException,
            is IllegalArgumentException -> null
            else -> throw e
        }
    }
