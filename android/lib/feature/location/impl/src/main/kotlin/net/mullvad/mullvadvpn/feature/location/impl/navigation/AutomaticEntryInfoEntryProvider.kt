package net.mullvad.mullvadvpn.feature.location.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.location.api.AutomaticEntryInfoNavKey
import net.mullvad.mullvadvpn.feature.location.impl.dialog.AutomaticEntryInfo

fun EntryProviderScope<NavKey2>.automaticEntryInfoEntry(navigator: Navigator) {
    entry<AutomaticEntryInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        AutomaticEntryInfo(navigator = navigator)
    }
}
