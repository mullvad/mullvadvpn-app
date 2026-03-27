package net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.apiaccess.api.DeleteApiAccessMethodNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.delete.DeleteApiAccessMethodConfirmation

internal fun EntryProviderScope<NavKey2>.deleteApiAccessEntry(navigator: Navigator) {
    entry<DeleteApiAccessMethodNavKey>(metadata = DialogSceneStrategy.dialog()) { navKey ->
        DeleteApiAccessMethodConfirmation(
            apiAccessMethodId = navKey.apiAccessMethodId,
            navigator = navigator,
        )
    }
}
