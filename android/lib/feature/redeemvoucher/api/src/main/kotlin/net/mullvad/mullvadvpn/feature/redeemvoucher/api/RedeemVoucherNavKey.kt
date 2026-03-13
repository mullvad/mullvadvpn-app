package net.mullvad.mullvadvpn.feature.redeemvoucher.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.NavResult

@Parcelize
object RedeemVoucherNavKey : NavKey2

@Parcelize
data class RedeemVoucherNavResult(val isTimeAdded: Boolean) : NavResult
