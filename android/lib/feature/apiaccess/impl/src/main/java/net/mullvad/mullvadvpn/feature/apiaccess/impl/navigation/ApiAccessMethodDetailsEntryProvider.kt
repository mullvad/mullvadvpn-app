package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.apiaccess.api.ApiAccessMethodDetailsNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.detail.ApiAccessMethodDetails

internal fun EntryProviderScope<NavKey2>.apiAccessMethodDetailsEntry(navigator: Navigator) {
    entry<ApiAccessMethodDetailsNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        ApiAccessMethodDetails(apiAccessMethodId = navKey.accessMethodId, navigator = navigator)
    }
}
