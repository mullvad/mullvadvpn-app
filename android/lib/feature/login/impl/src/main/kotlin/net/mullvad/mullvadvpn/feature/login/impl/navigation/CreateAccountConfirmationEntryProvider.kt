package net.mullvad.mullvadvpn.feature.login.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.CreateAccountConfirmationNavKey
import net.mullvad.mullvadvpn.feature.login.impl.CreateAccountConfirmation

internal fun EntryProviderScope<NavKey2>.createAccountConfirmationEntry(navigator: Navigator) {
    entry<CreateAccountConfirmationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        CreateAccountConfirmation(navigator = navigator)
    }
}
