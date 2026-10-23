package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.DomainFrontingInfoNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.info.DomainFrontingInfo

internal fun EntryProviderScope<NavKey2>.domainFrontingEntry(navigator: Navigator) {
    entry<DomainFrontingInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DomainFrontingInfo(navigator = navigator)
    }
}
