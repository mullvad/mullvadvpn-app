package net.mullvad.mullvadvpn.feature.deleteaccount.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountCompleteNavKey
import net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountcomplete.DeleteAccountComplete

internal fun EntryProviderScope<NavKey2>.deleteAccountCompleteEntry(navigator: Navigator) {
    entry<DeleteAccountCompleteNavKey>(metadata = slideInHorizontalTransition()) {
        DeleteAccountComplete(navigator = navigator)
    }
}
