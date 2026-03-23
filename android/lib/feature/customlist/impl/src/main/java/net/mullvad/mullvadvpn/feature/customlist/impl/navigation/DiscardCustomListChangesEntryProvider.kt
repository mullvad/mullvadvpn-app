package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.DiscardCustomListChangesNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.discard.DiscardChanges

internal fun EntryProviderScope<NavKey2>.discardCustomListChangesEntry(navigator: Navigator) {
    entry<DiscardCustomListChangesNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DiscardChanges(navigator = navigator)
    }
}
