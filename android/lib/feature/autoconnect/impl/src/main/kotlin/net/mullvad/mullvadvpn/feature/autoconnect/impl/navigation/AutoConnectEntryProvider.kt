package net.mullvad.mullvadvpn.feature.autoconnect.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.autoconnect.api.AutoConnectNavKey
import net.mullvad.mullvadvpn.feature.autoconnect.impl.AutoConnectAndLockdownMode

fun EntryProviderScope<NavKey2>.autoConnectEntry(navigator: Navigator) {
    entry<AutoConnectNavKey>(metadata = slideInHorizontalTransition()) {
        AutoConnectAndLockdownMode(navigator = navigator)
    }
}
