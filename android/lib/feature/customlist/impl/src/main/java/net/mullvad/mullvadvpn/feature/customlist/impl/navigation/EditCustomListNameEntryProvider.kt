package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNameNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.editname.EditCustomListName

internal fun EntryProviderScope<NavKey2>.editCustomListNameEntry(navigator: Navigator) {
    entry<EditCustomListNameNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        EditCustomListName(navArgs = navKey, navigator = navigator)
    }
}
