package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.EncryptedDnsProxyInfoNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.edpinfo.EncryptedDnsProxyInfo

internal fun EntryProviderScope<NavKey2>.encryptedDnsProxyAccessEntry(navigator: Navigator) {
    entry<EncryptedDnsProxyInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        EncryptedDnsProxyInfo(navigator = navigator)
    }
}
