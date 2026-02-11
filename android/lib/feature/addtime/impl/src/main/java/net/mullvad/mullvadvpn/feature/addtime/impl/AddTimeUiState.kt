package net.mullvad.mullvadvpn.feature.addtime.impl

import net.mullvad.mullvadvpn.lib.payment.model.ProductId

data class AddTimeUiState(
    val purchaseState: PurchaseState?,
    val billingPaymentState: PaymentState,
    val showSitePayment: Boolean,
    val tunnelStateBlocked: Boolean,
)

sealed interface PurchaseState {
    data object Connecting : PurchaseState

    data object VerificationStarted : PurchaseState

    data object VerifyingPurchase : PurchaseState

    data class Success(val productId: ProductId) : PurchaseState

    sealed interface Error : PurchaseState {
        data class TransactionIdError(val productId: ProductId) : Error

        data class OtherError(val productId: ProductId) : Error
    }
}
