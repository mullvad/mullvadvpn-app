package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableNavKey
import net.mullvad.mullvadvpn.feature.login.impl.apiunreachable.ApiUnreachableInfo

fun EntryProviderScope<NavKey>.apiUnreachableEntry(navigator: Navigator) {
    entry<ApiUnreachableNavKey> { navKey ->
        ApiUnreachableInfo(navigator = navigator, navArgs = navKey.args)
    }
}
