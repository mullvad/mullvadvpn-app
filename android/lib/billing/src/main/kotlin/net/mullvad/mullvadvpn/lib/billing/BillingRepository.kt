package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetailsResult
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryProductDetailsParams
import com.android.billingclient.api.QueryProductDetailsParams.Product
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.queryProductDetails
import com.android.billingclient.api.queryPurchasesAsync
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlin.coroutines.suspendCoroutine
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.lib.billing.extension.toBillingPurchase
import net.mullvad.mullvadvpn.lib.billing.extension.toPurchaseResult
import net.mullvad.mullvadvpn.lib.billing.extension.toQueryProductResultError
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryPurchasesResult

class BillingRepository(private val activity: Activity) {

    private val billingClient: BillingClient

    private val purchaseUpdateListener: PurchasesUpdatedListener =
        PurchasesUpdatedListener { result, purchases ->
            when (result.responseCode) {
                BillingResponseCode.OK -> {
                    purchases?.map(Purchase::toBillingPurchase)?.let { billingPurchases ->
                        _purchaseEvents.tryEmit(PurchaseEvent.PurchaseCompleted(billingPurchases))
                    }
                }
                BillingResponseCode.USER_CANCELED -> {
                    _purchaseEvents.tryEmit(PurchaseEvent.UserCanceled)
                }
                else -> {
                    _purchaseEvents.tryEmit(
                        PurchaseEvent.Error(
                            exception =
                                BillingException(
                                    responseCode = result.responseCode,
                                    message = result.debugMessage
                                )
                        )
                    )
                }
            }
        }

    private val _purchaseEvents = MutableSharedFlow<PurchaseEvent>(extraBufferCapacity = 1)
    val purchaseEvents = _purchaseEvents.asSharedFlow()

    init {
        billingClient =
            BillingClient.newBuilder(activity)
                .enablePendingPurchases()
                .setListener(purchaseUpdateListener)
                .build()
    }

    private suspend fun checkAndConnect() = suspendCoroutine {
        if (
            billingClient.isReady &&
                billingClient.connectionState == BillingClient.ConnectionState.CONNECTED
        ) {
            it.resume(Unit)
        } else {
            billingClient.startConnection(
                object : BillingClientStateListener {
                    override fun onBillingServiceDisconnected() {
                        // Maybe do something here?
                        it.resumeWithException(
                            BillingException(
                                BillingResponseCode.SERVICE_DISCONNECTED,
                                "Billing service disconnected"
                            )
                        )
                    }

                    override fun onBillingSetupFinished(result: BillingResult) {
                        if (result.responseCode == BillingResponseCode.OK) {
                            it.resume(Unit)
                        } else {
                            it.resumeWithException(
                                BillingException(result.responseCode, result.debugMessage)
                            )
                        }
                    }
                }
            )
        }
    }

    suspend fun queryProducts(productIds: List<String> = listOf(PRODUCT_ID)): QueryProductResult {
        val result = queryProductDetails(productIds)
        return when (result.billingResult.responseCode) {
            BillingResponseCode.OK ->
                (QueryProductResult.Ok(
                    products =
                        result.productDetailsList?.map { productDetails ->
                            BillingProduct(
                                productId = productDetails.productId,
                                price = productDetails.oneTimePurchaseOfferDetails?.formattedPrice
                                        ?: ""
                            )
                        }
                            ?: emptyList()
                ))
            else -> result.billingResult.toQueryProductResultError()
        }
    }

    suspend fun startPurchaseFlow(productId: String, transactionId: String): PurchaseFlowResult {
        return try {
            checkAndConnect()

            val productDetailsResult = queryProductDetails(listOf(productId))

            val productDetails =
                if (productDetailsResult.billingResult.responseCode == BillingResponseCode.OK) {
                    productDetailsResult.productDetailsList?.firstOrNull()
                        ?: return PurchaseFlowResult.ItemUnavailable
                } else {
                    throw BillingException(
                        responseCode = productDetailsResult.billingResult.responseCode,
                        message = productDetailsResult.billingResult.debugMessage
                    )
                }

            val productDetailsParamsList =
                listOf(
                    BillingFlowParams.ProductDetailsParams.newBuilder()
                        .setProductDetails(productDetails)
                        .build()
                )

            val billingFlowParams =
                BillingFlowParams.newBuilder()
                    .setProductDetailsParamsList(productDetailsParamsList)
                    .setObfuscatedAccountId(transactionId)
                    .build()

            // Launch the billing flow
            billingClient.launchBillingFlow(activity, billingFlowParams).toPurchaseResult()
        } catch (t: Throwable) {
            if (t is BillingException) {
                t.toBillingResult().toPurchaseResult()
            } else {
                PurchaseFlowResult.Error(
                    BillingException(BillingResponseCode.ERROR, t.message ?: "")
                )
            }
        }
    }

    suspend fun queryPurchases(productId: String = PRODUCT_ID): QueryPurchasesResult {
        return try {
            checkAndConnect()

            val queryPurchaseHistoryParams: QueryPurchasesParams =
                QueryPurchasesParams.newBuilder().setProductType(productId).build()

            val result = billingClient.queryPurchasesAsync(queryPurchaseHistoryParams)
            return when {
                result.purchasesList.isNotEmpty() ->
                    QueryPurchasesResult.PurchaseFound(
                        result.purchasesList.first().toBillingPurchase()
                    )
                result.billingResult.responseCode == BillingResponseCode.OK ->
                    QueryPurchasesResult.NoPurchasesFound
                else ->
                    QueryPurchasesResult.Error(
                        BillingException(
                            responseCode = result.billingResult.responseCode,
                            message = result.billingResult.debugMessage
                        )
                    )
            }
        } catch (t: Throwable) {
            if (t is BillingException) {
                QueryPurchasesResult.Error(t)
            } else {
                QueryPurchasesResult.Error(
                    exception =
                        BillingException(
                            responseCode = BillingResponseCode.ERROR,
                            message = t.message ?: ""
                        )
                )
            }
        }
    }

    private suspend fun queryProductDetails(productIds: List<String>): ProductDetailsResult {
        return try {
            checkAndConnect()

            val productList =
                productIds.map { productId ->
                    Product.newBuilder()
                        .setProductId(productId)
                        .setProductType(BillingClient.ProductType.INAPP)
                        .build()
                }
            val params = QueryProductDetailsParams.newBuilder()
            params.setProductList(productList)

            billingClient.queryProductDetails(params.build())
        } catch (t: Throwable) {
            if (t is BillingException) {
                return ProductDetailsResult(t.toBillingResult(), null)
            } else {
                return ProductDetailsResult(
                    BillingResult.newBuilder().setResponseCode(BillingResponseCode.ERROR).build(),
                    null
                )
            }
        }
    }

    companion object {
        private const val PRODUCT_ID = "test"
    }
}
