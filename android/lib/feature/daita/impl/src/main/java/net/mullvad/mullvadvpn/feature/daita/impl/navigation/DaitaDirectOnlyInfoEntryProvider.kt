package net.mullvad.mullvadvpn.feature.daita.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyInfoNavKey
import net.mullvad.mullvadvpn.feature.daita.impl.DaitaDirectOnlyInfo

fun EntryProviderScope<NavKey2>.daitaDirectOnlyInfoEntry(navigator: Navigator) {
    entry<DaitaDirectOnlyInfoNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DaitaDirectOnlyInfo(navigator = navigator)
    }
}
