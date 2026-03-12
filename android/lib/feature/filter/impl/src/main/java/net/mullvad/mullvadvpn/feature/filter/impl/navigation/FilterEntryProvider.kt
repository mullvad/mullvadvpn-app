package net.mullvad.mullvadvpn.feature.filter.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.filter.api.FilterNavKey
import net.mullvad.mullvadvpn.feature.filter.impl.Filter

fun EntryProviderScope<NavKey2>.filterEntry(navigator: Navigator) {
    entry<FilterNavKey> { Filter(navigator = navigator) }
}
