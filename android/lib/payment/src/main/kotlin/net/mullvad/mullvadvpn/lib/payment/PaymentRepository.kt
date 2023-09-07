package net.mullvad.mullvadvpn.lib.payment

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult

interface PaymentRepository {

    val purchaseResult: Flow<PurchaseResult>

    suspend fun purchaseBillingProduct(productId: String)

    suspend fun verifyPurchases()

    suspend fun queryPaymentAvailability(): PaymentAvailability
}
