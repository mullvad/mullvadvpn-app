package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.apiaccess.api.EditApiAccessMethodNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.edit.EditApiAccessMethod

internal fun EntryProviderScope<NavKey2>.editApiAccessMethodEntry(navigator: Navigator) {
    entry<EditApiAccessMethodNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        EditApiAccessMethod(apiAccessMethodId = navKey.accessMethodId, navigator = navigator)
    }
}
