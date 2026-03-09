package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.WelcomeNavKey
import net.mullvad.mullvadvpn.feature.home.impl.welcome.Welcome

internal fun EntryProviderScope<NavKey2>.welcomeEntry(navigator: Navigator) {
    entry<WelcomeNavKey> { Welcome(navigator = navigator) }
}
