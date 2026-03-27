package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableNavKey
import net.mullvad.mullvadvpn.feature.login.impl.apiunreachable.ApiUnreachableInfo

internal fun EntryProviderScope<NavKey2>.apiUnreachableEntry(navigator: Navigator) {
    entry<ApiUnreachableNavKey> { navKey ->
        ApiUnreachableInfo(navigator = navigator, navArgs = navKey)
    }
}
