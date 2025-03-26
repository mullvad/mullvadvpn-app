package net.mullvad.mullvadvpn.util

import android.content.pm.PackageManager
import android.graphics.drawable.Drawable

fun PackageManager.getApplicationIconOrNull(packageName: String): Drawable? =
    try {
        getApplicationIcon(packageName)
    } catch (e: PackageManager.NameNotFoundException) {
        // Name not found is thrown if the application is not installed
        null
    } catch (e: IllegalArgumentException) {
        // IllegalArgumentException is thrown if the application has an invalid icon
        null
    } catch (e: OutOfMemoryError) {
        // OutOfMemoryError is thrown if the icon is too large
        null
    }
