package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.Ipv6InfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.info.Ipv6Info

internal fun EntryProviderScope<NavKey2>.ipv6InfoEntry(navigator: Navigator) {
    entry<Ipv6InfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        Ipv6Info(navigator = navigator)
    }
}
