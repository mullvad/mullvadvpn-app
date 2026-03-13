package net.mullvad.mullvadvpn.feature.appinfo.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.appinfo.api.AppInfoNavKey
import net.mullvad.mullvadvpn.feature.appinfo.api.ChangelogNavKey
import net.mullvad.mullvadvpn.feature.appinfo.impl.AppInfo
import net.mullvad.mullvadvpn.feature.appinfo.impl.changelog.Changelog

fun EntryProviderScope<NavKey2>.changelogEntry(navigator: Navigator) {
    entry<ChangelogNavKey> { Changelog(navigator = navigator) }

    appInfoEntry(navigator)
}
