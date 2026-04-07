package net.mullvad.mullvadvpn.feature.addtime.impl.navigation

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.VerificationPendingNavKey
import net.mullvad.mullvadvpn.feature.addtime.impl.verificationpending.VerificationPending

@OptIn(ExperimentalMaterial3Api::class)
fun EntryProviderScope<NavKey2>.addTimeVerificationPendingEntry(navigator: Navigator) {
    entry<VerificationPendingNavKey>(metadata = DialogSceneStrategy.dialog()) {
        VerificationPending(navigator = navigator)
    }

    addTimeEntry(navigator)
}
