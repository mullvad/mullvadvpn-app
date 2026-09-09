package net.mullvad.mullvadvpn.lib.model

sealed interface RedeemVoucherError {
    data object InvalidVoucher : RedeemVoucherError

    data object VoucherAlreadyUsed : RedeemVoucherError

    data object TooShortVoucher : RedeemVoucherError

    data object EnteredAccountNumber : RedeemVoucherError

    data object ApiUnreachable : RedeemVoucherError

    data class Unknown(val error: Throwable) : RedeemVoucherError
}
