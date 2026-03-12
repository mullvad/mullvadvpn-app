package net.mullvad.mullvadvpn.feature.account.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.account.api.AccountNavKey
import net.mullvad.mullvadvpn.feature.account.impl.Account

fun EntryProviderScope<NavKey2>.accountEntry(navigator: Navigator) {
    entry<AccountNavKey> { Account(navigator = navigator) }
}
