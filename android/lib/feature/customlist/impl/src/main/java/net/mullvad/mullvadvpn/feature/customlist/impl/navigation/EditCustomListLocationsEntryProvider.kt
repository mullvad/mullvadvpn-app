package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListLocationsNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.editlocations.CustomListLocations

internal fun EntryProviderScope<NavKey2>.editCustomListLocationsEntry(navigator: Navigator) {
    entry<EditCustomListLocationsNavKey> { navKey ->
        CustomListLocations(navArgs = navKey, navigator = navigator)
    }
}
