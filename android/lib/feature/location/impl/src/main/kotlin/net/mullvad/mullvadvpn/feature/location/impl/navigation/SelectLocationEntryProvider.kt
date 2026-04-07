package net.mullvad.mullvadvpn.feature.location.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.topLevelTransition
import net.mullvad.mullvadvpn.feature.location.api.SelectLocationNavKey
import net.mullvad.mullvadvpn.feature.location.impl.SelectLocation

fun EntryProviderScope<NavKey2>.selectLocationEntry(navigator: Navigator) {
    entry<SelectLocationNavKey>(metadata = topLevelTransition()) {
        SelectLocation(navigator = navigator)
    }

    locationBottomSheetEntry(navigator)
    searchLocationEntry(navigator)
}
