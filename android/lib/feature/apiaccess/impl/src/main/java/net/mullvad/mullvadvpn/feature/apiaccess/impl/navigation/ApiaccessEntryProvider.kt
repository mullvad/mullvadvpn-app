package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.ApiAccessNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.list.ApiAccessList
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.list.ApiAccessListScreen

fun EntryProviderScope<NavKey2>.apiAccessEntry(navigator: Navigator) {
    entry<ApiAccessNavKey> {
        ApiAccessList(navigator = navigator)
    }
}
