package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.LoginNavKey
import net.mullvad.mullvadvpn.feature.login.impl.Login

fun EntryProviderScope<NavKey2>.loginEntry(navigator: Navigator) {
    entry<LoginNavKey> { navKey ->
        Login(navigator = navigator, accountNumber = navKey.accountNumber)
    }
}
