package net.mullvad.mullvadvpn.screen.splash.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.core.nav3.SplashNavKey
import net.mullvad.mullvadvpn.screen.splash.Splash

fun EntryProviderScope<NavKey>.splashEntry(navigator: Navigator) {
    entry<SplashNavKey> { Splash(navigator = navigator) }
}
