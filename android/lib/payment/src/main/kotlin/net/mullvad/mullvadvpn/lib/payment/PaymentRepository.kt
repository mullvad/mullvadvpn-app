package net.mullvad.mullvadvpn.lib.payment

import android.app.Activity
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.billing.BillingRepository
import net.mullvad.mullvadvpn.lib.billing.model.BillingPurchase
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryPurchasesResult

class PaymentRepository(
    activity: Activity,
    private val billingRepository: BillingRepository = BillingRepository(activity = activity),
    private val showWebPayment: Boolean
) {
    private val _billingPurchaseEvents =
        billingRepository.purchaseEvents.map { event ->
            when (event) {
                is PurchaseEvent.Error -> {
                    // Return error
                    PurchaseResult.PurchaseError
                }
                is PurchaseEvent.PurchaseCompleted -> {
                    // Verify towards api
                    if (verifyPurchase(event.purchases.first())) {
                        PurchaseResult.PurchaseCompleted
                    } else {
                        PurchaseResult.VerificationError
                    }
                }
                PurchaseEvent.UserCanceled -> {
                    // Purchase aborted
                    PurchaseResult.PurchaseCancelled
                }
            }
        }
    private val _purchaseResultEvents = MutableSharedFlow<PurchaseResult>(extraBufferCapacity = 1)
    val purchaseResult =
        combine(billingRepository.purchaseEvents, _purchaseResultEvents.asSharedFlow()) { event ->
            event
        }

    suspend fun queryAvailablePaymentTypes(): PaymentAvailability =
        PaymentAvailability(
            webPaymentAvailable = showWebPayment,
            billingPaymentAvailability = getBillingProducts()
        )

    suspend fun purchaseBillingProduct(productId: String) {
        // Get transaction id
        val transactionId =
            fetchTransactionId()
                ?: run {
                    _purchaseResultEvents.tryEmit(PurchaseResult.PurchaseError)
                    return
                }

        val result =
            billingRepository.startPurchaseFlow(
                productId = productId,
                transactionId = transactionId
            )

        if (result !is PurchaseFlowResult.Ok) {
            _purchaseResultEvents.tryEmit(PurchaseResult.PurchaseError)
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
                BillingPaymentAvailability.ProductsAvailable(
                    products = result.products.toPaymentProducts()
                )
            QueryProductResult.ItemUnavailable -> BillingPaymentAvailability.ProductsUnavailable
            QueryProductResult.BillingUnavailable ->
                BillingPaymentAvailability.Error.BillingUnavailable
            QueryProductResult.ServiceUnavailable ->
                BillingPaymentAvailability.Error.ServiceUnavailable
            is QueryProductResult.Error ->
                BillingPaymentAvailability.Error.Other(exception = result.exception)
        }

    private fun fetchTransactionId(): String? {
        // Placeholder function
        return "BOOPITOBOP"
    }

    private fun verifyPurchase(purchase: BillingPurchase): Boolean {
        // Placeholder function
        return true
    }
}
