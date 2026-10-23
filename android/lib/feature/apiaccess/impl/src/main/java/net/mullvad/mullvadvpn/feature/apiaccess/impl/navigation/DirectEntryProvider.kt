package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.DirectInfoNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.info.DirectInfo

internal fun EntryProviderScope<NavKey2>.directEntry(navigator: Navigator) {
    entry<DirectInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DirectInfo(navigator = navigator)
    }
}
