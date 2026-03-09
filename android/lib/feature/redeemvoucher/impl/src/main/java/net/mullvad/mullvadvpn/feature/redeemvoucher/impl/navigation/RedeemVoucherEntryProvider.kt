package net.mullvad.mullvadvpn.feature.redeemvoucher.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.scene.DialogSceneStrategy
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.redeemvoucher.api.RedeemVoucherNavKey
import net.mullvad.mullvadvpn.feature.redeemvoucher.impl.RedeemVoucher

fun EntryProviderScope<NavKey2>.redeemVoucherEntry(navigator: Navigator) {
    entry<RedeemVoucherNavKey>(metadata = DialogSceneStrategy.dialog()) {
        RedeemVoucher(navigator = navigator)
    }
}
