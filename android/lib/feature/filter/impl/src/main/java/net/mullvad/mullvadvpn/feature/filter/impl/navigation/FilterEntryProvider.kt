package net.mullvad.mullvadvpn.feature.filter.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.filter.api.FilterNavKey
import net.mullvad.mullvadvpn.feature.filter.impl.Filter

fun EntryProviderScope<NavKey2>.filterEntry(navigator: Navigator) {
    entry<FilterNavKey>(metadata = slideInHorizontalTransition()) { Filter(navigator = navigator) }
}
