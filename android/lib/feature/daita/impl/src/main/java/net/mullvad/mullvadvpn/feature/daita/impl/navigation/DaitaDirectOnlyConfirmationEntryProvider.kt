package net.mullvad.mullvadvpn.feature.daita.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyConfirmationNavKey
import net.mullvad.mullvadvpn.feature.daita.impl.DaitaDirectOnlyConfirmation

fun EntryProviderScope<NavKey2>.daitaDirectOnlyConfirmationEntry(navigator: Navigator) {
    entry<DaitaDirectOnlyConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        DaitaDirectOnlyConfirmation(navigator = navigator)
    }
}
