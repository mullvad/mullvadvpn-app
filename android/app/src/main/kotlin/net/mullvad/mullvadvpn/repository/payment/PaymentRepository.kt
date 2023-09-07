package net.mullvad.mullvadvpn.repository.payment

import net.mullvad.mullvadvpn.lib.billing.BillingRepository
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult

class PaymentRepository(
    private val billingRepository: BillingRepository,
    private val showWebPayment: Boolean
) {
    suspend fun queryAvailablePaymentTypes(): PaymentAvailability =
        PaymentAvailability(
            webPaymentAvailable = showWebPayment,
            billingPaymentAvailability = getBillingProducts()
        )

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
}
