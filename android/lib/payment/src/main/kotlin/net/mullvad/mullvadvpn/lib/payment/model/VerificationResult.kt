package net.mullvad.mullvadvpn.lib.payment.model

sealed interface VerificationResult {
    data object FetchingUnfinishedPurchases : VerificationResult

    data object VerificationStarted : VerificationResult

    // No verification was needed as there is no purchases to verify
    data object NoVerification : VerificationResult

    data object Success : VerificationResult

    // Generic error, add more cases as needed
    sealed interface Error : VerificationResult {
        data class BillingError(val exception: Throwable?) : Error

        data class VerificationError(val exception: Throwable?) : Error
    }
}
