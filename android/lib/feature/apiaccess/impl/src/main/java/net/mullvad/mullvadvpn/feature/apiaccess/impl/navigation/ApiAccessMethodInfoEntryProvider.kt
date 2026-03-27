package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.ApiAccessMethodInfoNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.info.ApiAccessMethodInfo

internal fun EntryProviderScope<NavKey2>.apiAccessMethodInfoEntry(navigator: Navigator) {
    entry<ApiAccessMethodInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ApiAccessMethodInfo(navigator = navigator)
    }
}
