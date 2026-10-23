package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.home.api.PauseVpnNavKey
import net.mullvad.mullvadvpn.feature.home.impl.connect.PauseVpnBottomSheet

internal fun EntryProviderScope<NavKey2>.pauseVpnEntry(navigator: Navigator) {
    entry<PauseVpnNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) {
        PauseVpnBottomSheet(navigator = navigator)
    }
}
