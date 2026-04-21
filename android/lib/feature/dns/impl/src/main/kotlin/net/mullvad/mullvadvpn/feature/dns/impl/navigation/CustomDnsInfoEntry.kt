package net.mullvad.mullvadvpn.feature.dns.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsInfoNavKey
import net.mullvad.mullvadvpn.feature.dns.impl.info.CustomDnsInfo

internal fun EntryProviderScope<NavKey2>.customDnsInfoEntry(navigator: Navigator) {
    entry<CustomDnsInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        CustomDnsInfo(navigator = navigator)
    }
}
