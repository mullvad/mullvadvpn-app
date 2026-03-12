package net.mullvad.mullvadvpn.feature.autoconnect.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.autoconnect.api.AutoConnectNavKey
import net.mullvad.mullvadvpn.feature.autoconnect.impl.AutoConnectAndLockdownMode

fun EntryProviderScope<NavKey2>.autoConnectEntry(navigator: Navigator) {
    entry<AutoConnectNavKey> {
        AutoConnectAndLockdownMode(navigator = navigator)
    }
}
