package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
sealed class RedeemVoucherError : Parcelable {
    data object InvalidVoucher : RedeemVoucherError()

    data object VoucherAlreadyUsed : RedeemVoucherError()

    data object RpcError : RedeemVoucherError()

    data class Unknown(val error: Throwable) : RedeemVoucherError()
}
