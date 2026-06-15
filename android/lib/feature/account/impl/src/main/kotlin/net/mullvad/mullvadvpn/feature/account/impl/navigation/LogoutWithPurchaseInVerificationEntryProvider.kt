package net.mullvad.mullvadvpn.feature.account.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.account.api.LogoutPurchaseVerificationNavKey
import net.mullvad.mullvadvpn.feature.account.impl.dialog.LogoutWithPurchaseInVerification

fun EntryProviderScope<NavKey2>.logoutWithPurchaseInVerificationEntry(navigator: Navigator) {
    entry<LogoutPurchaseVerificationNavKey>(metadata = DialogSceneStrategy.dialog()) {
        LogoutWithPurchaseInVerification(navigator = navigator)
    }
}
