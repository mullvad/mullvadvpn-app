package net.mullvad.mullvadvpn.applist

import android.content.pm.ApplicationInfo
import android.graphics.drawable.Drawable

data class AppInfo(val info: ApplicationInfo, val label: String) {
    var icon: Drawable? = null
}
