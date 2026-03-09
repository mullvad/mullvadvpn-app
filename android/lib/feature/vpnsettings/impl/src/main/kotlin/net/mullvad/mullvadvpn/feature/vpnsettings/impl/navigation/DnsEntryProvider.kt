package net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.vpnsettings.api.DnsNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.dns.Dns

internal fun EntryProviderScope<NavKey2>.dnsEntry(navigator: Navigator) {
    entry<DnsNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        Dns(navArgs = navKey, navigator = navigator)
    }
}
