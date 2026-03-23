package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.OutOfTimeNavKey
import net.mullvad.mullvadvpn.feature.home.impl.outoftime.OutOfTime

internal fun EntryProviderScope<NavKey2>.outOfTimeEntry(navigator: Navigator) {
    entry<OutOfTimeNavKey> { OutOfTime(navigator = navigator) }
}
