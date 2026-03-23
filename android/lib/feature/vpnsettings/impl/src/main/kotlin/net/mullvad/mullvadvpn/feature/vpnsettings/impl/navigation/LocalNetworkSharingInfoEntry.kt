package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.LocalNetworkSharingInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.info.LocalNetworkSharingInfo

internal fun EntryProviderScope<NavKey2>.localNetworkSharingInfoEntry(navigator: Navigator) {
    entry<LocalNetworkSharingInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        LocalNetworkSharingInfo(navigator = navigator)
    }
}
