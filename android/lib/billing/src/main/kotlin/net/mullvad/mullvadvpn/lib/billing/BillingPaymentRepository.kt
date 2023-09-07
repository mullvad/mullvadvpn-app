package net.mullvad.mullvadvpn.lib.billing

import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import net.mullvad.mullvadvpn.lib.billing.extension.toPaymentProducts
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult

class BillingPaymentRepository(private val billingRepository: BillingRepository) :
    PaymentRepository {
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
    override val purchaseResult: Flow<PurchaseResult> =
        merge(_billingPurchaseEvents, _purchaseResultEvents.asSharedFlow())

    override suspend fun queryPaymentAvailability(): PaymentAvailability = getBillingProducts()

    override suspend fun purchaseBillingProduct(productId: String) {
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

        if (result.responseCode != BillingResponseCode.OK) {
            _purchaseResultEvents.tryEmit(PurchaseResult.PurchaseError)
        }
    }

    override suspend fun verifyPurchases() {
        val result = billingRepository.queryPurchases()
        if (
            result.billingResult.responseCode == BillingResponseCode.OK &&
                result.purchasesList.isNotEmpty()
        ) {
            verifyPurchase(result.purchasesList.first())
        }
    }

    private suspend fun getBillingProducts(): PaymentAvailability {
        val result = billingRepository.queryProducts(listOf(ProductIds.OneMonth))
        return when {
            result.billingResult.responseCode == BillingResponseCode.OK &&
                result.productDetailsList.isNullOrEmpty() -> {
                PaymentAvailability.ProductsUnavailable
            }
            result.billingResult.responseCode == BillingResponseCode.OK ->
                PaymentAvailability.ProductsAvailable(
                    result.productDetailsList?.toPaymentProducts() ?: emptyList()
                )
            result.billingResult.responseCode == BillingResponseCode.BILLING_UNAVAILABLE ->
                PaymentAvailability.Error.BillingUnavailable
            result.billingResult.responseCode == BillingResponseCode.SERVICE_UNAVAILABLE ->
                PaymentAvailability.Error.ServiceUnavailable
            else ->
                PaymentAvailability.Error.Other(
                    BillingException(
                        result.billingResult.responseCode,
                        result.billingResult.debugMessage
                    )
                )
        }
    }

    private fun fetchTransactionId(): String? {
        // Placeholder function
        return "BOOPITOBOP"
    }

    private fun verifyPurchase(purchase: Purchase): Boolean {
        // Placeholder function
        return true
    }
}
