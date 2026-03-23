package net.mullvad.mullvadvpn.feature.customlist.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.impl.screen.editlist.EditCustomList

internal fun EntryProviderScope<NavKey2>.editCustomListEntry(navigator: Navigator) {
    entry<EditCustomListNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        EditCustomList(customListId = navKey.customListId, navigator = navigator)
    }
}
