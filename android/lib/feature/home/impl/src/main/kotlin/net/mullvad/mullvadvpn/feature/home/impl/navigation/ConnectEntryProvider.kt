package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.home.impl.connect.Connect

fun EntryProviderScope<NavKey>.connectEntry(navigator: Navigator) {
    entry<ConnectNavKey> {
        Connect(
            navigator = navigator,
            animatedVisibilityScope = LocalNavAnimatedContentScope.current,
            onNavigateToSelectLocation = {},
        )
    }
}
