package net.mullvad.mullvadvpn.feature.deleteaccount.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountConfirmationNavKey
import net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation.DeleteAccountConfirmation

internal fun EntryProviderScope<NavKey2>.deleteAccountConfirmationEntry(navigator: Navigator) {
    entry<DeleteAccountConfirmationNavKey>(metadata = slideInHorizontalTransition()) {
        DeleteAccountConfirmation(navigator = navigator)
    }
}
