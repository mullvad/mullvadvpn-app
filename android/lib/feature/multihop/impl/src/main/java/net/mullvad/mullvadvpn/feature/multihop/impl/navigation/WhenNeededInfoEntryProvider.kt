package net.mullvad.mullvadvpn.feature.multihop.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.multihop.api.WhenNeededInfoNavKey
import net.mullvad.mullvadvpn.feature.multihop.impl.WhenNeededInfo

fun EntryProviderScope<NavKey2>.whenNeededInfoEntry(navigator: Navigator) {
    entry<WhenNeededInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        WhenNeededInfo(navigator = navigator)
    }
}
