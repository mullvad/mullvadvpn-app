package net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.RemoveDeviceNavKey
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.RemoveDeviceConfirmation
import androidx.navigation3.scene.DialogSceneStrategy.Companion.DialogKey
import androidx.compose.ui.window.DialogProperties
import androidx.navigation3.scene.DialogSceneStrategy


fun EntryProviderScope<NavKey>.removeDeviceConfirmationDialogEntry(navigator: Navigator) {

    entry<RemoveDeviceNavKey>(
        metadata = DialogSceneStrategy.dialog()
    ) { navKey ->
        RemoveDeviceConfirmation(navigator = navigator, device = navKey.device)
    }
}
