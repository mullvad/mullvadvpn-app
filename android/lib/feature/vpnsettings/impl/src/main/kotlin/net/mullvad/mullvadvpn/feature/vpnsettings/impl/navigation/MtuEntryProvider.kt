package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.mtu.Mtu

internal fun EntryProviderScope<NavKey2>.mtuEntry(navigator: Navigator) {
    entry<MtuNavKey>(metadata = DialogSceneStrategy.dialog()) { navArgs ->
        Mtu(navArgs = navArgs, navigator = navigator)
    }
}
