package net.mullvad.mullvadvpn.usecase

import android.app.Activity
import arrow.core.Either
import arrow.core.right
import arrow.resilience.Schedule
import arrow.resilience.retryEither
import co.touchlab.kermit.Logger
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.transform
import net.mullvad.mullvadvpn.constant.VERIFICATION_BACK_OFF_FACTOR
import net.mullvad.mullvadvpn.constant.VERIFICATION_INITIAL_BACK_OFF_DURATION
import net.mullvad.mullvadvpn.constant.VERIFICATION_MAX_ATTEMPTS
import net.mullvad.mullvadvpn.lib.common.util.firstOrNullWithTimeout
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationError
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentUseCase {
    val paymentAvailability: Flow<PaymentAvailability?>
    val purchaseResult: Flow<PurchaseResult?>

    suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity)

    suspend fun queryPaymentAvailability()

    suspend fun resetPurchaseResult()

    suspend fun verifyPurchases(): Either<VerificationError, VerificationResult>

    suspend fun allAvailableProducts(): List<PaymentProduct>?
}

class PlayPaymentUseCase(private val paymentRepository: PaymentRepository) : PaymentUseCase {
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override val paymentAvailability = _paymentAvailability.asStateFlow()
    override val purchaseResult = _purchaseResult.asStateFlow()

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity) {
        paymentRepository
            .purchaseProduct(productId, activityProvider)
            .transform {
                emit(it)
                if (it.shouldDelayLoading()) {
                    delay(EXTRA_LOADING_DELAY_MS)
                }
            }
            .onEach { Logger.i("Purchase state: ${it::class.simpleName}") }
            .collect(_purchaseResult)
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun queryPaymentAvailability() {
        paymentRepository
            .queryPaymentAvailability()
            .onEach { Logger.i("Payment availability: ${it::class.simpleName}") }
            .collect(_paymentAvailability)
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun resetPurchaseResult() {
        _purchaseResult.emit(null)
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun verifyPurchases() =
        Schedule.exponential<VerificationError>(
                VERIFICATION_INITIAL_BACK_OFF_DURATION,
                VERIFICATION_BACK_OFF_FACTOR,
            )
            .and(Schedule.recurs(VERIFICATION_MAX_ATTEMPTS.toLong()))
            .retryEither { paymentRepository.verifyPurchases() }
            .onRight {
                if (it == VerificationResult.Success) {
                    // Update the payment availability after a successful verification.
                    queryPaymentAvailability()
                }
            }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun allAvailableProducts(): List<PaymentProduct>? =
        paymentRepository
            .queryPaymentAvailability()
            .filterIsInstance<PaymentAvailability.ProductsAvailable>()
            .firstOrNullWithTimeout(QUERY_PRODUCTS_TIMEOUT)
            ?.products

    private fun PurchaseResult?.shouldDelayLoading() =
        this is PurchaseResult.FetchingProducts || this is PurchaseResult.VerificationStarted

    companion object {
        const val EXTRA_LOADING_DELAY_MS = 300L
        const val QUERY_PRODUCTS_TIMEOUT = 3000L
    }
}

class EmptyPaymentUseCase : PaymentUseCase {
    override val paymentAvailability = MutableStateFlow(PaymentAvailability.ProductsUnavailable)
    override val purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity) {
        // No op
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun queryPaymentAvailability() {
        // No op
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun resetPurchaseResult() {
        // No op
    }

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun verifyPurchases() = VerificationResult.NothingToVerify.right()

    @Suppress("ensure every public functions method is named 'invoke' with operator modifier")
    override suspend fun allAvailableProducts(): List<PaymentProduct>? = null
}
