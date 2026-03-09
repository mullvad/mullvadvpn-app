package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.QuantumResistanceInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.info.QuantumResistanceInfo

internal fun EntryProviderScope<NavKey2>.quantumResistanceInfoEntry(navigator: Navigator) {
    entry<QuantumResistanceInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        QuantumResistanceInfo(navigator = navigator)
    }
}
