package net.mullvad.mullvadvpn.lib.repository

import android.app.Activity
import arrow.core.Either
import arrow.core.right
import arrow.resilience.Schedule
import arrow.resilience.retryEither
import co.touchlab.kermit.Logger
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.transform
import net.mullvad.mullvadvpn.lib.common.util.firstOrNullWithTimeout
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationError
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult

interface PaymentLogic {
    val paymentAvailability: Flow<PaymentAvailability?>
    val purchaseResult: Flow<PurchaseResult?>

    suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity)

    suspend fun queryPaymentAvailability()

    suspend fun resetPurchaseResult()

    suspend fun verifyPurchases(): Either<VerificationError, VerificationResult>

    suspend fun allAvailableProducts(): List<PaymentProduct>?
}

class PlayPaymentLogic(private val paymentRepository: PaymentRepository) : PaymentLogic {
    private val _paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val _purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    override val paymentAvailability = _paymentAvailability.asStateFlow()
    override val purchaseResult = _purchaseResult.asStateFlow()

    override suspend fun purchaseProduct(productId: ProductId, activityProvider: () -> Activity) {
        paymentRepository
            .purchaseProduct(productId, activityProvider)
            .transform {
                emit(it)
                if (it.shouldDelayLoading()) {
                    delay(EXTRA_LOADING_DELAY_MS)
                }
            }
            .onEach(::logPurchaseResult)
            .collect(_purchaseResult)
    }

    override suspend fun queryPaymentAvailability() {
        paymentRepository
            .queryPaymentAvailability()
            .onEach(::logPaymentAvailability)
            .collect(_paymentAvailability)
    }

    override suspend fun resetPurchaseResult() {
        _purchaseResult.emit(null)
    }

    override suspend fun verifyPurchases() =
        Schedule.exponential<VerificationError>(
                VERIFICATION_INITIAL_BACK_OFF_DURATION,
                VERIFICATION_BACK_OFF_FACTOR,
            )
            .and(Schedule.recurs(VERIFICATION_MAX_ATTEMPTS.toLong()))
            .doWhile { error, _ ->
                // If we have a verification error we should not retry as it will fail again.
                error !is VerificationError.PlayVerificationError.VerificationFailed
            }
            .retryEither { paymentRepository.verifyPurchases() }
            .onRight {
                if (it == VerificationResult.Success) {
                    // Update the payment availability after a successful verification.
                    queryPaymentAvailability()
                }
            }

    override suspend fun allAvailableProducts(): List<PaymentProduct>? =
        paymentRepository
            .queryPaymentAvailability()
            .filterIsInstance<PaymentAvailability.ProductsAvailable>()
            .firstOrNullWithTimeout(QUERY_PRODUCTS_TIMEOUT)
            ?.products

    private fun PurchaseResult?.shouldDelayLoading() =
        this is PurchaseResult.FetchingProducts || this is PurchaseResult.VerificationStarted

    private fun logPurchaseResult(result: PurchaseResult) {
        when (result) {
            PurchaseResult.Completed.Cancelled -> {
                Logger.i("Purchase cancelled")
            }
            is PurchaseResult.Completed -> {
                Logger.i("Purchase completed")
                if (result is PurchaseResult.Completed.Pending) {
                    Logger.i("Purchase is now pending")
                }
            }
            is PurchaseResult.Error -> {
                Logger.e("Purchase error")
                when (result) {
                    is PurchaseResult.Error.VerificationError ->
                        Logger.e("Could not verify purchase")
                    is PurchaseResult.Error.BillingError -> Logger.e("BillingError")
                    is PurchaseResult.Error.FetchProductsError ->
                        Logger.e("Could not fetch any products")
                    is PurchaseResult.Error.NoProductFound -> Logger.e("No product available")
                    is PurchaseResult.Error.TransactionIdError ->
                        Logger.e("Could not fetch transaction id")
                }
            }
            PurchaseResult.BillingFlowStarted -> Logger.i("Purchase flow started")
            PurchaseResult.FetchingObfuscationId -> Logger.i("Fetching obfuscation id...")
            PurchaseResult.FetchingProducts -> Logger.i("Fetching products...")
            PurchaseResult.VerificationStarted -> Logger.i("Verification started")
        }
    }

    private fun logPaymentAvailability(paymentAvailability: PaymentAvailability) {
        when (paymentAvailability) {
            is PaymentAvailability.Error -> {
                Logger.e("Unable to get products")
                when (paymentAvailability) {
                    PaymentAvailability.Error.BillingUnavailable -> Logger.e("BillingUnavailable")
                    PaymentAvailability.Error.DeveloperError -> Logger.e("DeveloperError")
                    PaymentAvailability.Error.FeatureNotSupported -> Logger.e("FeatureNotSupported")
                    PaymentAvailability.Error.ItemUnavailable -> Logger.e("ItemUnavailable")
                    is PaymentAvailability.Error.Other -> Logger.e("Other")
                    PaymentAvailability.Error.ServiceUnavailable -> Logger.e("ServiceUnavailable")
                }
            }
            PaymentAvailability.Loading -> Logger.i("Loading products...")
            PaymentAvailability.NoProductsFound -> Logger.e("No products found")
            is PaymentAvailability.ProductsAvailable -> Logger.i("Products available")
            PaymentAvailability.ProductsUnavailable -> Logger.i("Products unavailable")
        }
    }

    companion object {
        const val EXTRA_LOADING_DELAY_MS = 300L
        const val QUERY_PRODUCTS_TIMEOUT = 3000L

        const val VERIFICATION_MAX_ATTEMPTS = 4
        val VERIFICATION_INITIAL_BACK_OFF_DURATION = 3.seconds
        const val VERIFICATION_BACK_OFF_FACTOR = 3.toDouble()
    }
}

class EmptyPaymentUseCase : PaymentLogic {
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

    override suspend fun verifyPurchases() = VerificationResult.NothingToVerify.right()

    override suspend fun allAvailableProducts(): List<PaymentProduct>? = null
}
