package net.mullvad.mullvadvpn.feature.dns.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.ContentBlockersInfoNavKey
import net.mullvad.mullvadvpn.feature.dns.impl.info.ContentBlockersInfo

internal fun EntryProviderScope<NavKey2>.contentBlockersInfoEntry(navigator: Navigator) {
    entry<ContentBlockersInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        ContentBlockersInfo(navigator = navigator)
    }
}
