package net.mullvad.mullvadvpn.lib.model

sealed class RedeemVoucherError {
    data object InvalidVoucher : RedeemVoucherError()

    data object VoucherAlreadyUsed : RedeemVoucherError()

    data object RpcError : RedeemVoucherError()

    data class Unknown(val error: Throwable) : RedeemVoucherError()
}
