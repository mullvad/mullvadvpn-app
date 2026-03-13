package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CustomListNavKey

fun EntryProviderScope<NavKey2>.customListEntry(navigator: Navigator) {
    entry<CustomListNavKey> {
//        CustomLists(navigator = navigator)
    }

    createCustomListEntry(navigator)
    deleteCustomListConfirmationEntry(navigator)
    deleteCustomListEntry(navigator)
    editCustomListEntry(navigator)
    editCustomListNameEntry(navigator)
    discardCustomListChangesEntry(navigator)
}
