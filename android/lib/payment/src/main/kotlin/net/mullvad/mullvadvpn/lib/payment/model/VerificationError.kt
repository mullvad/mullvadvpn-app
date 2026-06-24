package net.mullvad.mullvadvpn.lib.payment.model

sealed interface VerificationError {
    data class BillingError(val errorCode: Int, val exception: Throwable) : VerificationError

    sealed interface PlayVerificationError : VerificationError {
        data object VerificationFailed : PlayVerificationError

        data object Other : PlayVerificationError
    }
}
