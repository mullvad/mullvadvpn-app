package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ResetServerIpOverrideConfirmationNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.reset.ResetServerIpOverridesConfirmation

internal fun EntryProviderScope<NavKey2>.resetServerIpOverrideConfirmationEntry(
    navigator: Navigator
) {
    entry<ResetServerIpOverrideConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ResetServerIpOverridesConfirmation(navigator = navigator)
    }
}
