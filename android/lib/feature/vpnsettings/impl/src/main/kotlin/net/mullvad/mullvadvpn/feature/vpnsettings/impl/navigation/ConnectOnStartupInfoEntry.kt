package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.ConnectOnStartupInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.info.ConnectOnStartupInfo

internal fun EntryProviderScope<NavKey2>.connectOnStartupInfoEntry(navigator: Navigator) {
    entry<ConnectOnStartupInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ConnectOnStartupInfo(navigator = navigator)
    }
}
