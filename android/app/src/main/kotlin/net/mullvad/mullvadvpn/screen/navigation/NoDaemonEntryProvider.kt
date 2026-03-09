package net.mullvad.mullvadvpn.screen.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.screen.nodaemon.NoDaemon

fun EntryProviderScope<NavKey2>.noDaemonEntry(navigator: Navigator) {
    entry<NoDaemonNavKey> { NoDaemon(navigator = navigator) }
}
