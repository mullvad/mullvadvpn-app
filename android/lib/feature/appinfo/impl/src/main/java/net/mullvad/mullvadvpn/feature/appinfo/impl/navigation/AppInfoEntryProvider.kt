package net.mullvad.mullvadvpn.feature.appinfo.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.appinfo.api.AppInfoNavKey
import net.mullvad.mullvadvpn.feature.appinfo.impl.AppInfo

internal fun EntryProviderScope<NavKey2>.appInfoEntry(navigator: Navigator) {
    entry<AppInfoNavKey>(metadata = slideInHorizontalTransition()) {
        AppInfo(navigator = navigator)
    }
}
