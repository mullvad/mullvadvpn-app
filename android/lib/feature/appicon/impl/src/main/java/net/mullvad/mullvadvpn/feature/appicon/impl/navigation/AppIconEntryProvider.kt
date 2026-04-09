package net.mullvad.mullvadvpn.feature.appicon.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.core.scene.ListDetailSceneStrategy
import net.mullvad.mullvadvpn.feature.appicon.api.AppIconNavKey
import net.mullvad.mullvadvpn.feature.appicon.impl.AppIcon

fun EntryProviderScope<NavKey2>.appIconEntry(navigator: Navigator) {
    entry<AppIconNavKey>(
        metadata = slideInHorizontalTransition()
    ) {
        AppIcon(navigator = navigator)
    }
}
