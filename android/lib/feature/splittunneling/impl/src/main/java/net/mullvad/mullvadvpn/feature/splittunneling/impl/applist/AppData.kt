package net.mullvad.mullvadvpn.feature.splittunneling.impl.applist

data class AppData(
    val packageName: String,
    val iconRes: Int,
    val name: String,
    val isSystemApp: Boolean = false,
)
