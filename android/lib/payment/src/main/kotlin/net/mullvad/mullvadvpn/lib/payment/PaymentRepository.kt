package net.mullvad.mullvadvpn.lib.payment

import kotlinx.coroutines.flow.singleOrNull
import net.mullvad.mullvadvpn.lib.billing.BillingRepository
import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct
import net.mullvad.mullvadvpn.lib.billing.model.BillingPurchase
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryPurchasesResult

class PaymentRepository(
    private val billingRepository: BillingRepository,
    private val showWebPayment: Boolean
) {
    suspend fun queryAvailablePaymentTypes(): PaymentAvailability =
        PaymentAvailability(
            webPaymentAvailable = showWebPayment,
            billingPaymentAvailability = getBillingProducts()
        )

    suspend fun purchaseBillingProduct(product: BillingProduct): PurchaseResult {
        // Get transaction id
        val transactionId = fetchTransactionId()

        val result =
            billingRepository.startPurchaseFlow(
                productId = product.productId,
                transactionId = transactionId
            )

        if (result is PurchaseFlowResult.Ok) {
            // Wait for events
            return when (val purchaseEvent = billingRepository.purchaseEvents.singleOrNull()) {
                is PurchaseEvent.Error -> {
                    // Return error
                    PurchaseResult.PurchaseError
                }
                is PurchaseEvent.PurchaseCompleted -> {
                    // Verify towards api
                    if (verifyPurchase(purchaseEvent.purchases.first())) {
                        PurchaseResult.PurchaseCompleted
                    } else {
                        PurchaseResult.VerificationError
                    }
                }
                PurchaseEvent.UserCanceled -> {
                    // Purchase aborted
                    PurchaseResult.PurchaseCancelled
                }
                null -> {
                    // Return error
                    PurchaseResult.PurchaseError
                }
            }
        } else {
            // Return error
            return PurchaseResult.PurchaseError
        }
    }

    suspend fun verifyPurchases() {
        val result = billingRepository.queryPurchases()
        if (result is QueryPurchasesResult.PurchaseFound) {
            verifyPurchase(result.purchase)
        }
    }

    private suspend fun getBillingProducts(): BillingPaymentAvailability =
        when (val result = billingRepository.queryProducts()) {
            is QueryProductResult.Ok ->
                BillingPaymentAvailability.ProductsAvailable(products = result.products)
            QueryProductResult.ItemUnavailable -> BillingPaymentAvailability.ProductsUnavailable
            QueryProductResult.BillingUnavailable ->
                BillingPaymentAvailability.Error.BillingUnavailable
            QueryProductResult.ServiceUnavailable ->
                BillingPaymentAvailability.Error.ServiceUnavailable
            is QueryProductResult.Error ->
                BillingPaymentAvailability.Error.Other(exception = result.exception)
        }

    private fun fetchTransactionId(): String {
        // Placeholder function
        return "BOOPITOBOP"
    }

    private fun verifyPurchase(purchase: BillingPurchase): Boolean {
        // Placeholder function
        return true
    }
}
