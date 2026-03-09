package net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.DeviceListNavKey
import net.mullvad.mullvadvpn.feature.login.impl.Login

fun EntryProviderScope<NavKey>.deviceListEntry(navigator: Navigator) {
    entry<DeviceListNavKey> { Login(navigator = navigator) }
}
