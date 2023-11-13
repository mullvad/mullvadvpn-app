package net.mullvad.mullvadvpn.lib.payment

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentRepository {

    fun purchaseProduct(productId: String) : Flow<PurchaseResult>

    fun verifyPurchases() : Flow<VerificationResult>

    fun queryPaymentAvailability(): Flow<PaymentAvailability>
}
