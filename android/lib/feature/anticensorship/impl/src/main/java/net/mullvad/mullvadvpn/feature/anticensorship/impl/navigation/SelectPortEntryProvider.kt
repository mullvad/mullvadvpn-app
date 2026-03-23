package net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.anticensorship.api.SelectPortNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.impl.selectport.SelectPort

internal fun EntryProviderScope<NavKey2>.selectPortEntry(navigator: Navigator) {
    entry<SelectPortNavKey>(metadata = slideInHorizontalTransition()) { navArgs ->
        SelectPort(navArgs = navArgs, navigator = navigator)
    }
}
