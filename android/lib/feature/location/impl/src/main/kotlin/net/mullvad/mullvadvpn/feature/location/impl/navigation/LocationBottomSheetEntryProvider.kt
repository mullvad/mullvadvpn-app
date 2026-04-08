package net.mullvad.mullvadvpn.feature.location.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetNavKey
import net.mullvad.mullvadvpn.feature.location.impl.bottomsheet.LocationBottomSheets

internal fun EntryProviderScope<NavKey2>.locationBottomSheetEntry(navigator: Navigator) {
    entry<LocationBottomSheetNavKey>(metadata = SingleOverlaySceneStrategy.overlay()) { navKey ->
        LocationBottomSheets(navigator = navigator, locationBottomSheetState = navKey.state)
    }
}
