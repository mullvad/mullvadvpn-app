package net.mullvad.mullvadvpn.feature.managedevices.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.managedevices.api.ManageDevicesNavKey
import net.mullvad.mullvadvpn.feature.managedevices.impl.ManageDevices

fun EntryProviderScope<NavKey2>.manageDevicesEntry(navigator: Navigator) {
    entry<ManageDevicesNavKey> { navKey ->
        ManageDevices(accountNumber = navKey.accountNumber, navigator = navigator)
    }

    manageDevicesRemoveConfirmationEntry(navigator)
}
