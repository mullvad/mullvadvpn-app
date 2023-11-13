package net.mullvad.mullvadvpn.lib.billing

import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.Purchase
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flow
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
        return@flow
    }

    override fun purchaseProduct(productId: ProductId): Flow<PurchaseResult> = flow {
        emit(PurchaseResult.PurchaseStarted)
        // Get transaction id
        val obfuscatedId: String =
            when (val result = initialisePurchase()) {
                is PlayPurchaseInitResult.Ok -> result.obfuscatedId
                else -> {
                    emit(PurchaseResult.Error.TransactionIdError(null))
                    return@flow
                }
            }

        val result =
            billingRepository.startPurchaseFlow(
                productId = productId.id,
                obfuscatedId = obfuscatedId
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
            is PurchaseEvent.PurchaseCompleted -> {
                val purchase =
                    event.purchases.firstOrNull()
                        ?: run {
                            emit(PurchaseResult.Error.BillingError(null))
                            return@flow
                        }
                if (purchase.purchaseState == Purchase.PurchaseState.PENDING) {
                    emit(PurchaseResult.PurchasePending)
                } else {
                    emit(PurchaseResult.VerificationStarted)
                    if (verifyPurchase(event.purchases.first()) == PlayPurchaseVerifyResult.Ok) {
                        emit(PurchaseResult.PurchaseCompleted)
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
        val purchases = billingRepository.queryPurchases()
        when {
            purchases.billingResult.responseCode == BillingResponseCode.OK &&
                purchases.purchasesList.isNotEmpty() -> {
                emit(VerificationResult.VerificationStarted)
                val verificationResult = verifyPurchase(purchases.purchasesList.first())
                emit(
                    when (verificationResult) {
                        is PlayPurchaseVerifyResult.Error ->
                            VerificationResult.Error.VerificationError(null)
                        PlayPurchaseVerifyResult.Ok -> VerificationResult.Success
                    }
                )
            }
            purchases.billingResult.responseCode == BillingResponseCode.OK ->
                emit(VerificationResult.NoVerification)
            else ->
                emit(
                    VerificationResult.Error.BillingError(
                        BillingException(
                            purchases.billingResult.responseCode,
                            purchases.billingResult.debugMessage
                        )
                    )
                )
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
