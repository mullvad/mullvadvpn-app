package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.apiaccess.api.ApiAccessNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.list.ApiAccessList

fun EntryProviderScope<NavKey2>.apiAccessEntry(navigator: Navigator) {
    entry<ApiAccessNavKey>(metadata = slideInHorizontalTransition()) {
        ApiAccessList(navigator = navigator)
    }

    apiAccessMethodDetailsEntry(navigator)
    apiAccessMethodInfoEntry(navigator)
    editApiAccessMethodEntry(navigator)
    deleteApiAccessEntry(navigator)
    discardApiAccessChangesEntry(navigator)
    encryptedDnsProxyAccessEntry(navigator)
    saveApiAccessMethodEntry(navigator)
}
