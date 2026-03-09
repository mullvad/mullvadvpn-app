package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.CreateAccountConfirmationNavKey
import net.mullvad.mullvadvpn.feature.login.impl.CreateAccountConfirmation

fun EntryProviderScope<NavKey>.createAccountConfirmationEntry(navigator: Navigator) {
    entry<CreateAccountConfirmationNavKey> { CreateAccountConfirmation(navigator = navigator) }
}
