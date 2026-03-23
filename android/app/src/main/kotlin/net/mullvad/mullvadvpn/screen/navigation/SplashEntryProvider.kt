package net.mullvad.mullvadvpn.screen.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.screen.splash.Splash

fun EntryProviderScope<NavKey2>.splashEntry(navigator: Navigator) {
    entry<SplashNavKey> { Splash(navigator = navigator) }
}
