package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.DiscardCustomListChangesNavKey

fun EntryProviderScope<NavKey2>.discardCustomListEntry(navigator: Navigator) {
    entry<DiscardCustomListChangesNavKey> {
//        Customlist(navigator = navigator)
    }
}
