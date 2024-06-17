package net.mullvad.mullvadvpn.lib.payment.model

sealed interface VerificationError {
    data class BillingError(val exception: Throwable) : VerificationError

    data object PlayVerificationError : VerificationError
}
