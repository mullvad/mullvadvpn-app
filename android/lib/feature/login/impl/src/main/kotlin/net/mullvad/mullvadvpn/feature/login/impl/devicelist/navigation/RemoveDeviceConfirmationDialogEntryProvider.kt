package net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.login.api.RemoveDeviceNavKey
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.RemoveDeviceConfirmation

fun EntryProviderScope<NavKey2>.removeDeviceConfirmationDialogEntry(navigator: Navigator) {
    entry<RemoveDeviceNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        RemoveDeviceConfirmation(navigator = navigator, device = navKey.device)
    }
}
