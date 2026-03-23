package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.DeviceNameInfoNavKey
import net.mullvad.mullvadvpn.feature.home.impl.welcome.DeviceNameInfo

internal fun EntryProviderScope<NavKey2>.deviceNameInfoEntry(navigator: Navigator) {
    entry<DeviceNameInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DeviceNameInfo(navigator = navigator)
    }
}
