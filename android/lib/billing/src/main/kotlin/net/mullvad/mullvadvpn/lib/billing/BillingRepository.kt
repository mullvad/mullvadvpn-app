package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetailsResult
import com.android.billingclient.api.PurchasesResult
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

class BillingRepository(private val activity: Activity) {

    private val billingClient: BillingClient

    private val purchaseUpdateListener: PurchasesUpdatedListener =
        PurchasesUpdatedListener { result, purchases ->
            when (result.responseCode) {
                BillingResponseCode.OK -> {
                    purchases?.let {
                        _purchaseEvents.tryEmit(PurchaseEvent.PurchaseCompleted(purchases))
                    }
                }
                BillingResponseCode.USER_CANCELED -> {
                    _purchaseEvents.tryEmit(PurchaseEvent.UserCanceled)
                }
                else -> {
                    _purchaseEvents.tryEmit(PurchaseEvent.Error(result))
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

    suspend fun queryProducts(): ProductDetailsResult {
        return try {
            checkAndConnect()

            val productList = ArrayList<Product>()
            productList.add(
                Product.newBuilder()
                    .setProductId(PRODUCT_ID)
                    .setProductType(BillingClient.ProductType.INAPP)
                    .build()
            )
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

    suspend fun startPurchaseFlow(
        productDetails: ProductDetails,
        transactionId: String
    ): BillingResult {
        return try {
            checkAndConnect()

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
            billingClient.launchBillingFlow(activity, billingFlowParams)
        } catch (t: Throwable) {
            if (t is BillingException) {
                t.toBillingResult()
            } else {
                BillingResult.newBuilder()
                    .setResponseCode(BillingResponseCode.ERROR)
                    .setDebugMessage(t.message ?: "")
                    .build()
            }
        }
    }

    suspend fun queryPurchases(): PurchasesResult {
        return try {
            checkAndConnect()

            val queryPurchaseHistoryParams: QueryPurchasesParams =
                QueryPurchasesParams.newBuilder().setProductType(PRODUCT_ID).build()

            billingClient.queryPurchasesAsync(queryPurchaseHistoryParams)
        } catch (t: Throwable) {
            if (t is BillingException) {
                PurchasesResult(t.toBillingResult(), emptyList())
            } else {
                PurchasesResult(
                    BillingResult.newBuilder().setResponseCode(BillingResponseCode.ERROR).build(),
                    emptyList()
                )
            }
        }
    }

    companion object {
        private const val PRODUCT_ID = "test"
    }
}
