package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.loginTransition
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.home.api.OutOfTimeNavKey
import net.mullvad.mullvadvpn.feature.home.api.WelcomeNavKey
import net.mullvad.mullvadvpn.feature.login.api.DeviceListNavKey
import net.mullvad.mullvadvpn.feature.login.api.LoginNavKey
import net.mullvad.mullvadvpn.feature.login.impl.Login

fun EntryProviderScope<NavKey2>.loginEntry(navigator: Navigator) {

    entry<LoginNavKey>(
        metadata =
            loginTransition {
                // Fade out if we are navigating to one of the following
                when (navigator.backStack.dropLast(1).lastOrNull()) {
                    OutOfTimeNavKey,
                    WelcomeNavKey,
                    ConnectNavKey,
                    is DeviceListNavKey -> true
                    else -> false
                }
            }
    ) { navKey ->
        Login(navigator = navigator, accountNumber = navKey.accountNumber)
    }

    apiUnreachableEntry(navigator)
    createAccountConfirmationEntry(navigator)
}
