package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.DiscardApiAccessChangesNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.discardchanges.DiscardApiAccessChanges

internal fun EntryProviderScope<NavKey2>.discardApiAccessChangesEntry(navigator: Navigator) {
    entry<DiscardApiAccessChangesNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DiscardApiAccessChanges(navigator = navigator)
    }
}
