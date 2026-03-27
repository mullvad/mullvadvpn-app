package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideInfoNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.info.ServerIpOverridesInfo

fun EntryProviderScope<NavKey2>.serverIpOverrideInfoEntry(navigator: Navigator) {
    entry<ServerIpOverrideInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ServerIpOverridesInfo(navigator = navigator)
    }
}
