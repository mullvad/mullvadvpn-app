package net.mullvad.mullvadvpn.feature.redeemvoucher.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult

@Parcelize object RedeemVoucherNavKey : NavKey2

@Parcelize data class RedeemVoucherNavResult(val isTimeAdded: Boolean) : NavResult
