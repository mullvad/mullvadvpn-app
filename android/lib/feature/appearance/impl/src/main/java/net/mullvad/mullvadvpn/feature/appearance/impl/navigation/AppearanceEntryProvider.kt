package net.mullvad.mullvadvpn.feature.appearance.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.appearance.api.AppearanceNavKey
import net.mullvad.mullvadvpn.feature.appearance.impl.Appearance

fun EntryProviderScope<NavKey2>.appearanceEntry(navigator: Navigator) {
    entry<AppearanceNavKey> { Appearance(navigator = navigator) }
}
