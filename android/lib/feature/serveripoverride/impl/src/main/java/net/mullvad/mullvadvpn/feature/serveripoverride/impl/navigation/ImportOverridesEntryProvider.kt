package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverridesNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.ImportOverridesBottomSheet

internal fun EntryProviderScope<NavKey2>.importOverridesEntry(navigator: Navigator) {
    entry<ImportOverridesNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) {
        ImportOverridesBottomSheet(navigator = navigator, overridesActive = true)
    }
}
