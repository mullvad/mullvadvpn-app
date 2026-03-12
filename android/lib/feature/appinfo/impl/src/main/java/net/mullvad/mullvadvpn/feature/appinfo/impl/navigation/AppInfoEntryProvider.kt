package net.mullvad.mullvadvpn.feature.appinfo.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.appinfo.api.AppInfoNavKey
import net.mullvad.mullvadvpn.feature.appinfo.impl.AppInfo

fun EntryProviderScope<NavKey2>.appInfoEntry(navigator: Navigator) {
    entry<AppInfoNavKey> { AppInfo(navigator = navigator) }
}
