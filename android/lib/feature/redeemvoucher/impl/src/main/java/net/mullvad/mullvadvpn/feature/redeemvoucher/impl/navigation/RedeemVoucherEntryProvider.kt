package net.mullvad.mullvadvpn.feature.redeemvoucher.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.redeemvoucher.api.RedeemVoucherNavKey
import net.mullvad.mullvadvpn.feature.redeemvoucher.impl.RedeemVoucher

fun EntryProviderScope<NavKey2>.redeemVoucherEntry(navigator: Navigator) {
    entry<RedeemVoucherNavKey> {
        RedeemVoucher(navigator = navigator)
    }
}
