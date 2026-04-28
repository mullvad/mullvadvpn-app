package net.mullvad.mullvadvpn.lib.payment.model

import net.mullvad.mullvadvpn.lib.payment.model.VerificationError.PlayVerificationError

sealed interface PurchaseResult {
    data object FetchingProducts : PurchaseResult

    data object FetchingObfuscationId : PurchaseResult

    data object BillingFlowStarted : PurchaseResult

    data object VerificationStarted : PurchaseResult

    sealed interface Completed : PurchaseResult {
        data class Success(val productId: ProductId) : Completed

        data object Cancelled : Completed

        // This ends our part of the purchase flow. The rest is handled by Google and the api.
        data class Pending(val productId: ProductId) : Completed
    }

    sealed interface Error : PurchaseResult {
        data class NoProductFound(val productId: ProductId) : Error

        data class FetchProductsError(val productId: ProductId, val exception: Throwable?) : Error

        data class TransactionIdError(val productId: ProductId, val exception: Throwable?) : Error

        data class BillingError(val exception: Throwable?) : Error

        sealed interface VerificationError : Error {
            data object VerificationFailed : VerificationError
            data object Other : VerificationError
        }
    }

    fun isTerminatingState(): Boolean = this is Completed || this is Error
}
