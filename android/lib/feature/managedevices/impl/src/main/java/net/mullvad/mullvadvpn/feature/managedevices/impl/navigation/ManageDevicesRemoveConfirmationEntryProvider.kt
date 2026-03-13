package net.mullvad.mullvadvpn.feature.managedevices.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.managedevices.api.ManageDevicesRemoveConfirmationNavKey
import net.mullvad.mullvadvpn.feature.managedevices.impl.confirmation.ManageDevicesRemoveConfirmation

internal fun EntryProviderScope<NavKey2>.manageDevicesRemoveConfirmationEntry(navigator: Navigator) {
    entry<ManageDevicesRemoveConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey
        ->
        ManageDevicesRemoveConfirmation(navigator = navigator, device = navKey.device)
    }
}
