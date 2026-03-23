package net.mullvad.mullvadvpn.feature.deleteaccount.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountNavKey
import net.mullvad.mullvadvpn.feature.deleteaccount.impl.DeleteAccount

fun EntryProviderScope<NavKey2>.deleteAccountEntry(navigator: Navigator) {
    entry<DeleteAccountNavKey>(metadata = slideInHorizontalTransition()) {
        DeleteAccount(navigator = navigator)
    }

    deleteAccountCompleteEntry(navigator)
    deleteAccountConfirmationEntry(navigator)
}
