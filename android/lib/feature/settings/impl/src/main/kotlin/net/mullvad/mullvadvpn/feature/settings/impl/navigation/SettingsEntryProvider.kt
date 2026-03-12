package net.mullvad.mullvadvpn.feature.settings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.feature.settings.impl.Settings

fun EntryProviderScope<NavKey2>.settingsEntry(navigator: Navigator) {
    entry<SettingsNavKey> { Settings(navigator = navigator) }
}
