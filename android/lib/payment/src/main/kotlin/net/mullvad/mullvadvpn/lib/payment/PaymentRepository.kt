package net.mullvad.mullvadvpn.lib.payment

import android.app.Activity
import arrow.core.Either
import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationError
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentRepository {

    fun purchaseProduct(
        productId: ProductId,
        activityProvider: () -> Activity,
    ): Flow<PurchaseResult>

    suspend fun verifyPurchases(): Either<VerificationError, VerificationResult>

    fun queryPaymentAvailability(): Flow<PaymentAvailability>
}
