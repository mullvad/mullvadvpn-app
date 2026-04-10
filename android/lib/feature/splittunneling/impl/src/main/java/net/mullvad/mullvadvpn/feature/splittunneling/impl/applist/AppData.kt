package net.mullvad.mullvadvpn.feature.splittunneling.impl.applist

import net.mullvad.mullvadvpn.lib.model.PackageName

data class AppData(
    val packageName: PackageName,
    val iconRes: Int,
    val name: String,
    val isSystemApp: Boolean = false,
)
