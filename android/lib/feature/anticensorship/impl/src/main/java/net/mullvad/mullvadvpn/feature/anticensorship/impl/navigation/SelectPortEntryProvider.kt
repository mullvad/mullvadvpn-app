package net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.SelectPortNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.impl.selectport.SelectPort

internal fun EntryProviderScope<NavKey2>.selectPortEntry(navigator: Navigator) {
    entry<SelectPortNavKey> {
        SelectPort(navigator = navigator)
    }
}
