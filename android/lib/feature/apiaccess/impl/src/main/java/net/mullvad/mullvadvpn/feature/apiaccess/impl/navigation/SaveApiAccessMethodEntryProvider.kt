package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.SaveApiAccessMethodNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.save.SaveApiAccessMethod

internal fun EntryProviderScope<NavKey2>.saveApiAccessMethodEntry(navigator: Navigator) {
    entry<SaveApiAccessMethodNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        SaveApiAccessMethod(navArgs = navKey, navigator = navigator)
    }
}
