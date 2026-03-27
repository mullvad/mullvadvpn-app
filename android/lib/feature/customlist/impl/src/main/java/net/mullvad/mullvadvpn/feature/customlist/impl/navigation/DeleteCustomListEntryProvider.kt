package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.DeleteCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.delete.DeleteCustomList

internal fun EntryProviderScope<NavKey2>.deleteCustomListEntry(navigator: Navigator) {
    entry<DeleteCustomListNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        DeleteCustomList(navArgs = navKey, navigator = navigator)
    }
}
