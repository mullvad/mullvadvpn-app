package net.mullvad.mullvadvpn.feature.appinfo.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.slideInHorizontalTransition
import net.mullvad.mullvadvpn.feature.appinfo.api.ChangelogNavKey
import net.mullvad.mullvadvpn.feature.appinfo.impl.changelog.Changelog

fun EntryProviderScope<NavKey2>.changelogEntry(navigator: Navigator) {
    entry<ChangelogNavKey>(metadata = slideInHorizontalTransition()) { navKey ->
        Changelog(navArgs = navKey, navigator = navigator)
    }

    appInfoEntry(navigator)
}
