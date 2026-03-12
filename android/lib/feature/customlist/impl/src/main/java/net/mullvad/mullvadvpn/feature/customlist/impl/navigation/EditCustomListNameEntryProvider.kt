package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNameNavKey

fun EntryProviderScope<NavKey>.editCustomListNameEntry(navigator: Navigator) {
    entry<EditCustomListNameNavKey> {
//        Customlist(navigator = navigator)
    }
}
