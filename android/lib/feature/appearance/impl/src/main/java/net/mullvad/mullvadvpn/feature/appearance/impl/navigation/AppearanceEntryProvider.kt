package net.mullvad.mullvadvpn.feature.appearance.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.appearance.api.AppearanceNavKey
import net.mullvad.mullvadvpn.feature.appearance.impl.Appearance

fun EntryProviderScope<NavKey2>.appearanceEntry(navigator: Navigator) {
    entry<AppearanceNavKey>(metadata = slideInHorizontalTransition()) {
        Appearance(navigator = navigator)
    }
}
