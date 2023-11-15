package net.mullvad.mullvadvpn.usecase

import android.app.Activity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentUseCase {
    val paymentAvailability: Flow<PaymentAvailability?>
    val purchaseResult: Flow<PurchaseResult?>

    suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity)

    suspend fun queryPaymentAvailability()

    suspend fun resetPurchaseResult()

    suspend fun verifyPurchases()
}

class PlayPaymentUseCase(private val paymentRepository: PaymentRepository) : PaymentUseCase {
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override val paymentAvailability = _paymentAvailability.asStateFlow()
    override val purchaseResult = _purchaseResult.asStateFlow()

    override suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity) {
        paymentRepository.purchaseProduct(productId, activityProvider).collect(_purchaseResult)
    }

    override suspend fun queryPaymentAvailability() {
        paymentRepository.queryPaymentAvailability().collect(_paymentAvailability)
    }

    override suspend fun resetPurchaseResult() {
        _purchaseResult.emit(null)
    }

    override suspend fun verifyPurchases() {
        paymentRepository.verifyPurchases().collect {
            if (it == VerificationResult.Success) {
                // Update the payment availability after a successful verification.
                queryPaymentAvailability()
            }
        }
    }
}

class EmptyPaymentUseCase : PaymentUseCase {
    override val paymentAvailability = MutableStateFlow(PaymentAvailability.ProductsUnavailable)
    override val purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity) {
        // No op
    }

    override suspend fun queryPaymentAvailability() {
        // No op
    }

    override suspend fun resetPurchaseResult() {
        // No op
    }

    override suspend fun verifyPurchases() {
        // No op
    }
}
