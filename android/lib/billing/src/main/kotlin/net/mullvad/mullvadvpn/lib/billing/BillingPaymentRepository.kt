package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flow
import net.mullvad.mullvadvpn.lib.billing.extension.getProductDetails
import net.mullvad.mullvadvpn.lib.billing.extension.purchases
import net.mullvad.mullvadvpn.lib.billing.extension.responseCode
import net.mullvad.mullvadvpn.lib.billing.extension.toBillingException
import net.mullvad.mullvadvpn.lib.billing.extension.toPaymentAvailability
import net.mullvad.mullvadvpn.lib.billing.extension.toPaymentStatus
import net.mullvad.mullvadvpn.lib.billing.extension.toPurchaseResult
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult

class BillingPaymentRepository(
    private val billingRepository: BillingRepository,
    private val playPurchaseRepository: PlayPurchaseRepository
) : PaymentRepository {

    override fun queryPaymentAvailability(): Flow<PaymentAvailability> = flow {
        emit(PaymentAvailability.Loading)
        val purchases = billingRepository.queryPurchases()
        val productIdToPaymentStatus =
            purchases.purchasesList
                .filter { it.products.isNotEmpty() }
                .associate { it.products.first() to it.purchaseState.toPaymentStatus() }
        emit(
            billingRepository
                .queryProducts(listOf(ProductIds.OneMonth))
                .toPaymentAvailability(productIdToPaymentStatus)
        )
    }

    override fun purchaseProduct(
        productId: ProductId,
        activityProvider: () -> Activity
    ): Flow<PurchaseResult> = flow {
        emit(PurchaseResult.FetchingProducts)

        val productDetailsResult = billingRepository.queryProducts(listOf(productId.value))

        val productDetails =
            when (productDetailsResult.billingResult.responseCode) {
                BillingResponseCode.OK -> {
                    productDetailsResult.getProductDetails(productId.value)
                        ?: run {
                            emit(PurchaseResult.Error.NoProductFound(productId))
                            return@flow
                        }
                }
                else -> {
                    emit(
                        PurchaseResult.Error.FetchProductsError(
                            productId,
                            productDetailsResult.toBillingException()
                        )
                    )
                    return@flow
                }
            }

        // Get transaction id
        emit(PurchaseResult.FetchingObfuscationId)
        val obfuscatedId: String =
            when (val result = initialisePurchase()) {
                is PlayPurchaseInitResult.Ok -> result.obfuscatedId
                else -> {
                    emit(PurchaseResult.Error.TransactionIdError(productId, null))
                    return@flow
                }
            }

        val result =
            billingRepository.startPurchaseFlow(
                productDetails = productDetails,
                obfuscatedId = obfuscatedId,
                activityProvider = activityProvider
            )

        if (result.responseCode == BillingResponseCode.OK) {
            emit(PurchaseResult.BillingFlowStarted)
        } else {
            emit(
                PurchaseResult.Error.BillingError(
                    BillingException(result.responseCode, result.debugMessage)
                )
            )
            return@flow
        }

        // Wait for a callback from the billing library
        when (val event = billingRepository.purchaseEvents.firstOrNull()) {
            is PurchaseEvent.Error -> emit(event.toPurchaseResult())
            is PurchaseEvent.Completed -> {
                val purchase =
                    event.purchases.firstOrNull()
                        ?: run {
                            emit(PurchaseResult.Error.BillingError(null))
                            return@flow
                        }
                if (purchase.purchaseState == Purchase.PurchaseState.PENDING) {
                    emit(PurchaseResult.Completed.Pending)
                } else {
                    emit(PurchaseResult.VerificationStarted)
                    if (verifyPurchase(event.purchases.first()) == PlayPurchaseVerifyResult.Ok) {
                        emit(PurchaseResult.Completed.Success)
                    } else {
                        emit(PurchaseResult.Error.VerificationError(null))
                    }
                }
            }
            PurchaseEvent.UserCanceled -> emit(event.toPurchaseResult())
            else -> emit(PurchaseResult.Error.BillingError(null))
        }
    }

    override fun verifyPurchases(): Flow<VerificationResult> = flow {
        emit(VerificationResult.FetchingUnfinishedPurchases)
        val purchasesResult = billingRepository.queryPurchases()
        when (purchasesResult.responseCode()) {
            BillingResponseCode.OK -> {
                val purchases = purchasesResult.purchases()
                if (purchases.isNotEmpty()) {
                    emit(VerificationResult.VerificationStarted)
                    val verificationResult = verifyPurchase(purchases.first())
                    emit(
                        when (verificationResult) {
                            is PlayPurchaseVerifyResult.Error ->
                                VerificationResult.Error.VerificationError(null)
                            PlayPurchaseVerifyResult.Ok -> VerificationResult.Success
                        }
                    )
                } else {
                    emit(VerificationResult.NothingToVerify)
                }
            }
            else ->
                emit(VerificationResult.Error.BillingError(purchasesResult.toBillingException()))
        }
    }

    private suspend fun initialisePurchase(): PlayPurchaseInitResult {
        return playPurchaseRepository.initializePlayPurchase()
    }

    private suspend fun verifyPurchase(purchase: Purchase): PlayPurchaseVerifyResult {
        return playPurchaseRepository.verifyPlayPurchase(
            PlayPurchase(
                productId = purchase.products.first(),
                purchaseToken = purchase.purchaseToken,
            )
        )
    }
}
