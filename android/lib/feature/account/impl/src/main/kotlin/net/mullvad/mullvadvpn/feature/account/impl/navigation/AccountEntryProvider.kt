package net.mullvad.mullvadvpn.feature.account.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.accountTransition
import net.mullvad.mullvadvpn.feature.account.api.AccountNavKey
import net.mullvad.mullvadvpn.feature.account.impl.Account

fun EntryProviderScope<NavKey2>.accountEntry(navigator: Navigator) {
    entry<AccountNavKey>(metadata = accountTransition()) { Account(navigator = navigator) }
}
