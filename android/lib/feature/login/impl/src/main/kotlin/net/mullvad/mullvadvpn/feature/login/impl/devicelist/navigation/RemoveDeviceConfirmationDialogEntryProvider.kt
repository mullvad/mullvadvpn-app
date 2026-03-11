package net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.DeviceListNavKey
import net.mullvad.mullvadvpn.feature.login.api.RemoveDeviceNavKey
import net.mullvad.mullvadvpn.feature.login.impl.Login
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.DeviceList
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.RemoveDeviceConfirmation

fun EntryProviderScope<NavKey>.removeDeviceConfirmationDialogEntry(navigator: Navigator) {
    entry<RemoveDeviceNavKey> { navKey ->
        RemoveDeviceConfirmation(navigator = navigator, device = navKey.device)
    }
}
