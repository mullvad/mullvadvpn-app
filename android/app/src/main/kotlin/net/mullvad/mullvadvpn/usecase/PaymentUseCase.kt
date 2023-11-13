package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.constant.PAYMENT_AVAILABILITY_DEBOUNCE
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.extensions.toPurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentUseCase {
    val paymentAvailability: Flow<PaymentAvailability?>
    val purchaseResult: Flow<PurchaseResult?>

    suspend fun purchaseProduct(productId: String)

    suspend fun queryPaymentAvailability()

    fun resetPurchaseResult()

    suspend fun verifyPurchases(updatePurchaseResult: Boolean)
}

class PlayPaymentUseCase(private val paymentRepository: PaymentRepository) : PaymentUseCase {
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override val paymentAvailability = _paymentAvailability.asStateFlow()
    override val purchaseResult = _purchaseResult.asStateFlow()

    override suspend fun purchaseProduct(productId: String) {
        paymentRepository?.purchaseProduct(productId)?.collect(_purchaseResult)
    }

    @OptIn(FlowPreview::class)
    override suspend fun queryPaymentAvailability() {
        paymentRepository
            .queryPaymentAvailability()
            .debounce(PAYMENT_AVAILABILITY_DEBOUNCE) // This is added to avoid flickering
            .collect(_paymentAvailability)
    }

    override fun resetPurchaseResult() {
        _purchaseResult.tryEmit(null)
    }

    override suspend fun verifyPurchases(updatePurchaseResult: Boolean) {
        if (updatePurchaseResult) {
            paymentRepository
                .verifyPurchases()
                .map(VerificationResult::toPurchaseResult)
                .collect(_purchaseResult)
        } else {
            paymentRepository.verifyPurchases().collect {
                if (it == VerificationResult.Success) {
                    // Update the payment availability after a successful verification.
                    queryPaymentAvailability()
                }
            }
        }
    }
}

class EmptyPaymentUseCase : PaymentUseCase {
    override val paymentAvailability = MutableStateFlow(PaymentAvailability.ProductsUnavailable)
    override val purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override suspend fun purchaseProduct(productId: String) {
        // No op
    }

    override suspend fun queryPaymentAvailability() {
        // No op
    }

    override fun resetPurchaseResult() {
        // No op
    }

    override suspend fun verifyPurchases(updatePurchaseResult: Boolean) {
        // No op
    }
}
