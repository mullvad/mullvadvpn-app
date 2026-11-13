package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import android.content.Context
import co.touchlab.kermit.Logger
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
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
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.model.PlayExternalObfuscatedAccountId

class BillingRepository(context: Context) {
    private val purchaseUpdateListener: PurchasesUpdatedListener =
        PurchasesUpdatedListener { result, purchases ->
            when (result.responseCode) {
                BillingResponseCode.OK -> {
                    _purchaseEvents.tryEmit(PurchaseEvent.Completed(purchases?.toList().orEmpty()))
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
                                    subResponseCode = result.onPurchasesUpdatedSubResponseCode,
                                    message = result.debugMessage,
                                ),
                        ),
                    )
                }
            }
        }

    private val billingClientConnection = BillingClientConnection(context, purchaseUpdateListener)

    private val _purchaseEvents = MutableSharedFlow<PurchaseEvent>(extraBufferCapacity = 1)
    val purchaseEvents = _purchaseEvents.asSharedFlow()

    suspend fun queryProducts(productIds: List<String>): ProductDetailsResult {
        return queryProductDetails(productIds)
    }

    suspend fun startPurchaseFlow(
        productDetails: ProductDetails,
        obfuscatedId: PlayExternalObfuscatedAccountId,
        activityProvider: () -> Activity,
    ): BillingResult {
        return try {
            if (obfuscatedId.value.isEmpty()) {
                Logger.e("Obfuscated id is empty")
                return BillingResult.newBuilder().setResponseCode(BillingResponseCode.ERROR).build()
            }

            val productDetailsParamsList =
                listOf(
                    BillingFlowParams.ProductDetailsParams.newBuilder()
                        .setProductDetails(productDetails)
                        .build(),
                )

            val billingFlowParams =
                BillingFlowParams.newBuilder()
                    .setProductDetailsParamsList(productDetailsParamsList)
                    .setObfuscatedAccountId(obfuscatedId.value)
                    .build()

            val activity = activityProvider()
            // Launch the billing flow
            billingClientConnection.getClient().launchBillingFlow(activity, billingFlowParams)
        } catch (t: BillingException) {
            t.toBillingResult()
        }
    }

    suspend fun queryPurchases(): PurchasesResult {
        return try {
            val queryPurchaseHistoryParams: QueryPurchasesParams =
                QueryPurchasesParams.newBuilder()
                    .setProductType(BillingClient.ProductType.INAPP)
                    .build()

            billingClientConnection.getClient().queryPurchasesAsync(queryPurchaseHistoryParams)
        } catch (t: BillingException) {
            t.toPurchasesResult()
        }
    }

    fun endConnection() {
        billingClientConnection.endConnection()
    }

    @Suppress("TooGenericExceptionCaught")
    private suspend fun queryProductDetails(productIds: List<String>): ProductDetailsResult {
        return try {
            val productList = productIds.map { productId ->
                Product.newBuilder()
                    .setProductId(productId)
                    .setProductType(BillingClient.ProductType.INAPP)
                    .build()
            }
            val params = QueryProductDetailsParams.newBuilder()
            params.setProductList(productList)

            billingClientConnection.getClient().queryProductDetails(params.build())
        } catch (billingException: BillingException) {
            ProductDetailsResult(billingException.toBillingResult(), null)
        } catch (_: Throwable) {
            ProductDetailsResult(
                BillingResult.newBuilder().setResponseCode(BillingResponseCode.ERROR).build(),
                null,
            )
        }
    }
}
