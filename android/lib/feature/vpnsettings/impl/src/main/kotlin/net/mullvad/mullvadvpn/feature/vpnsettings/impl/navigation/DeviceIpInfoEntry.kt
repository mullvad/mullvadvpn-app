package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.DeviceIpInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.info.DeviceIpInfo

internal fun EntryProviderScope<NavKey2>.deviceIpInfoEntry(navigator: Navigator) {
    entry<DeviceIpInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DeviceIpInfo(navigator = navigator)
    }
}
