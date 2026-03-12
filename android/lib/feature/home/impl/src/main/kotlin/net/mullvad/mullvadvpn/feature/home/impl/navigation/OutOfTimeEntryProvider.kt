package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.home.api.OutOfTimeNavKey
import net.mullvad.mullvadvpn.feature.home.impl.outoftime.OutOfTime

fun EntryProviderScope<NavKey2>.outOfTimeEntry(navigator: Navigator) {
    entry<OutOfTimeNavKey> {
        OutOfTime(
            navigator = navigator
        )
    }
}
