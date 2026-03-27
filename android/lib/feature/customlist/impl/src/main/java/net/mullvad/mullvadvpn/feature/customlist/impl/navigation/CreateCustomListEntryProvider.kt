package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CreateCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.create.CreateCustomList

internal fun EntryProviderScope<NavKey2>.createCustomListEntry(navigator: Navigator) {
    entry<CreateCustomListNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        CreateCustomList(locationCode = navKey.locationCode, navigator = navigator)
    }
}
