package net.mullvad.mullvadvpn.lib.payment

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentRepository {

    fun purchaseBillingProduct(productId: String) : Flow<PurchaseResult>

    suspend fun verifyPurchases() : Flow<VerificationResult>

    suspend fun queryPaymentAvailability(): Flow<PaymentAvailability>
}
