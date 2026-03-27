package net.mullvad.mullvadvpn.feature.settings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.settings.api.FaqRemoteNavKey
import net.mullvad.mullvadvpn.feature.settings.impl.FaqRemote

internal fun EntryProviderScope<NavKey2>.faqRemoteEntry(navigator: Navigator) {
    entry<FaqRemoteNavKey>() {
        FaqRemote(navigator = navigator)
    }
}
