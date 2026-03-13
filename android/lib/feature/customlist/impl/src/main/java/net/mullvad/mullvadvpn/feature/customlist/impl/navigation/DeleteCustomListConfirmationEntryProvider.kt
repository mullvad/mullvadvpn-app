package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.DeleteCustomListConfirmationNavKey

internal fun EntryProviderScope<NavKey2>.deleteCustomListConfirmationEntry(navigator: Navigator) {
    entry<DeleteCustomListConfirmationNavKey> {
//        Customlist(navigator = navigator)
    }
}
