package net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.CustomPortNavKey
import net.mullvad.mullvadvpn.feature.anticensorship.impl.customport.CustomPort

fun EntryProviderScope<NavKey2>.customPortEntry(navigator: Navigator) {
    entry<CustomPortNavKey>(
        metadata = DialogSceneStrategy.dialog()
    ) { navKey ->
        CustomPort(navArg = navKey, navigator = navigator)
    }
}
