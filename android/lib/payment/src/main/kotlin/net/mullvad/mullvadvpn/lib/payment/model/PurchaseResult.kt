package net.mullvad.mullvadvpn.lib.payment.model

sealed interface PurchaseResult {
    data object PurchaseStarted : PurchaseResult

    data object BillingFlowStarted : PurchaseResult

    data object VerificationStarted : PurchaseResult

    data object PurchaseCompleted : PurchaseResult

    data object PurchasePending : PurchaseResult

    data object PurchaseCancelled : PurchaseResult

    sealed interface Error : PurchaseResult {
        data class TransactionIdError(val exception: Throwable?) : Error

        data class BillingError(val exception: Throwable?) : Error

        data class VerificationError(val exception: Throwable?) : Error
    }

    fun isTerminatingState(): Boolean =
        this is PurchaseCompleted || this is PurchaseCancelled || this is Error
}
