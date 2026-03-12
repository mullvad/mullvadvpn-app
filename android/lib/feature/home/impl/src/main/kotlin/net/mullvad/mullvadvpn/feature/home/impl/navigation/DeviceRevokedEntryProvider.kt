package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.ui.LocalNavAnimatedContentScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.home.api.DeviceRevokedNavKey
import net.mullvad.mullvadvpn.feature.home.impl.connect.Connect
import net.mullvad.mullvadvpn.feature.home.impl.devicerevoked.DeviceRevoked

fun EntryProviderScope<NavKey2>.deviceRevokedEntry(navigator: Navigator) {
    entry<DeviceRevokedNavKey> {
        DeviceRevoked(navigator = navigator)
    }
}
