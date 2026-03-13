package net.mullvad.mullvadvpn.feature.location.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.location.api.SelectLocationNavKey
import net.mullvad.mullvadvpn.feature.location.impl.SelectLocation

fun EntryProviderScope<NavKey2>.selectLocationEntry(navigator: Navigator) {
    entry<SelectLocationNavKey> { SelectLocation(navigator = navigator) }

    searchLocationEntry(navigator)
}
