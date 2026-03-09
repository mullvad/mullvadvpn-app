package net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByTextNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.importbytext.ImportOverridesByText

internal fun EntryProviderScope<NavKey2>.importOverrideByTextScreenEntry(navigator: Navigator) {
    entry<ImportOverrideByTextNavKey> { ImportOverridesByText(navigator = navigator) }
}
