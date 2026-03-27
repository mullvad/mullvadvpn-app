package net.mullvad.mullvadvpn.feature.settings.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.topLevelTransition
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.feature.settings.impl.Settings

fun EntryProviderScope<NavKey2>.settingsEntry(navigator: Navigator) {
    entry<SettingsNavKey>(metadata = topLevelTransition()) { Settings(navigator = navigator) }

    faqRemoteEntry(navigator)
}
