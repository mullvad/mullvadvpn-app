package net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.login.api.DeviceListNavKey
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.DeviceList

fun EntryProviderScope<NavKey2>.deviceListEntry(navigator: Navigator) {
    entry<DeviceListNavKey> { navKey ->
        DeviceList(accountNumber = navKey.accountNumber, navigator = navigator)
    }
}
