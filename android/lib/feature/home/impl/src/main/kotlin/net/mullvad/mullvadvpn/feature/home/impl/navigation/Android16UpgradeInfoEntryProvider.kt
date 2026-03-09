package net.mullvad.mullvadvpn.feature.home.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.Android16UpgradeInfoNavKey
import net.mullvad.mullvadvpn.feature.home.impl.connect.Android16UpgradeWarningInfo

internal fun EntryProviderScope<NavKey2>.android16UpgradeInfoEntry(navigator: Navigator) {
    entry<Android16UpgradeInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        Android16UpgradeWarningInfo(navigator = navigator)
    }
}
