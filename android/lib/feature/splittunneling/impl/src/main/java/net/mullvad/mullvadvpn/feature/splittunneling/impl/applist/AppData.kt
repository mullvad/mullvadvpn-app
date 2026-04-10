package net.mullvad.mullvadvpn.feature.splittunneling.impl.applist

import net.mullvad.mullvadvpn.lib.model.AppId

data class AppData(
    val packageName: AppId,
    val iconRes: Int,
    val name: String,
    val isSystemApp: Boolean = false,
)
