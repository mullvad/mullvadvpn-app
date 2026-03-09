package net.mullvad.mullvadvpn.feature.login.impl.apiunreachable.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableNavKey
import net.mullvad.mullvadvpn.feature.login.impl.Login

fun EntryProviderScope<NavKey>.deviceListEntry(navigator: Navigator) {
    entry<ApiUnreachableNavKey> { Login(navigator = navigator) }
}
