package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import android.content.Context
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
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
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import org.koin.core.component.KoinComponent

class BillingRepository(context: Context) : KoinComponent {

    private val billingClient: BillingClient

    private val purchaseUpdateListener: PurchasesUpdatedListener =
        PurchasesUpdatedListener { result, purchases ->
            when (result.responseCode) {
                BillingResponseCode.OK -> {
                    _purchaseEvents.tryEmit(
                        PurchaseEvent.PurchaseCompleted(purchases?.toList() ?: emptyList())
                    )
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
            BillingClient.newBuilder(context)
                .enablePendingPurchases()
                .setListener(purchaseUpdateListener)
                .build()
    }

    private val checkAndConnectMutex = Mutex()

    private suspend fun checkAndConnect() =
        checkAndConnectMutex.withLock {
            suspendCoroutine {
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
        }

    suspend fun queryProducts(productIds: List<String> = listOf(PRODUCT_ID)): ProductDetailsResult {
        return queryProductDetails(productIds)
    }

    suspend fun startPurchaseFlow(productId: String, transactionId: String): BillingResult {
        return try {
            checkAndConnect()

            val productDetailsResult = queryProductDetails(listOf(productId))

            val productDetails =
                if (productDetailsResult.billingResult.responseCode == BillingResponseCode.OK) {
                    productDetailsResult.productDetailsList?.firstOrNull()
                        ?: throw BillingException(
                            responseCode = productDetailsResult.billingResult.responseCode,
                            message = productDetailsResult.billingResult.debugMessage
                        )
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

            // Get the activity from Koin
            val activity =
                getKoin().getOrNull<Activity>()
                    ?: throw BillingException(
                        responseCode = BillingResponseCode.ERROR,
                        message = "No activity found"
                    )
            // Launch the billing flow
            billingClient.launchBillingFlow(activity, billingFlowParams)
        } catch (t: Throwable) {
            if (t is BillingException) {
                t.toBillingResult()
            } else {
                throw t
            }
        }
    }

    suspend fun queryPurchases(productId: String = PRODUCT_ID): PurchasesResult {
        return try {
            checkAndConnect()

            val queryPurchaseHistoryParams: QueryPurchasesParams =
                QueryPurchasesParams.newBuilder().setProductType(productId).build()

            billingClient.queryPurchasesAsync(queryPurchaseHistoryParams)
        } catch (t: Throwable) {
            if (t is BillingException) {
                t.toPurchasesResult()
            } else {
                throw t
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
