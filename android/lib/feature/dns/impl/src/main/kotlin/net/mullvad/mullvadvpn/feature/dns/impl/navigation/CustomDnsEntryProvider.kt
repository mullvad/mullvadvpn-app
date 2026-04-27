package net.mullvad.mullvadvpn.feature.dns.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavKey
import net.mullvad.mullvadvpn.feature.dns.impl.CustomDns

internal fun EntryProviderScope<NavKey2>.customDnsEntry(navigator: Navigator) {
    entry<CustomDnsNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        CustomDns(navArgs = navKey, navigator = navigator)
    }
}
