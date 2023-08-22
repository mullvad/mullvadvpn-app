package net.mullvad.mullvadvpn.lib.billing

import android.app.Activity
import app.cash.turbine.test
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetailsResult
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesResult
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.queryProductDetails
import com.android.billingclient.api.queryPurchasesAsync
import io.mockk.CapturingSlot
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.billing.extension.toBillingPurchase
import net.mullvad.mullvadvpn.lib.billing.model.BillingException
import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct
import net.mullvad.mullvadvpn.lib.billing.model.BillingPurchase
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryPurchasesResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class BillingRepositoryTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockActivity: Activity = mockk()
    private lateinit var billingRepository: BillingRepository

    private val mockBillingClientBuilder: BillingClient.Builder = mockk(relaxed = true)
    private val mockBillingClient: BillingClient = mockk()

    private val listenerSlot: CapturingSlot<PurchasesUpdatedListener> = CapturingSlot()

    @Before
    fun setUp() {
        mockkStatic(BILLING_CLIENT_CLASS)
        mockkStatic(BILLING_CLIENT_KOTLIN_CLASS)
        mockkStatic(BILLING_FLOW_PARAMS)
        mockkStatic(PURCHASE_EXTENSION)

        every { BillingClient.newBuilder(any()) } returns mockBillingClientBuilder
        every { mockBillingClientBuilder.enablePendingPurchases() } returns mockBillingClientBuilder
        every { mockBillingClientBuilder.setListener(capture(listenerSlot)) } returns
            mockBillingClientBuilder
        every { mockBillingClientBuilder.build() } returns mockBillingClient

        billingRepository = BillingRepository(mockActivity)
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun testQueryProductsOk() = runTest {
        // Arrange
        val mockBillingResult: BillingResult = mockk()
        val mockProductDetails: ProductDetails = mockk()
        val mockProductDetailsResult: ProductDetailsResult = mockk()
        val productId = "TEST"
        val price = "44.4"
        val expectedBillingProduct = listOf(BillingProduct(productId = productId, price = price))

        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryProductDetails(any()) } returns mockProductDetailsResult
        every { mockProductDetailsResult.billingResult } returns mockBillingResult
        every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
        every { mockProductDetails.productId } returns productId
        every { mockProductDetails.oneTimePurchaseOfferDetails?.formattedPrice } returns price

        // Act
        val result = billingRepository.queryProducts()

        // Assert
        assertIs<QueryProductResult.Ok>(result)
        assertEquals(expectedBillingProduct, result.products)
    }

    @Test
    fun testQueryProductsItemUnavailable() = runTest {
        // Arrange
        val mockBillingResult: BillingResult = mockk()
        val mockProductDetailsResult: ProductDetailsResult = mockk()

        every { mockBillingResult.responseCode } returns BillingResponseCode.ITEM_UNAVAILABLE
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryProductDetails(any()) } returns mockProductDetailsResult
        every { mockProductDetailsResult.billingResult } returns mockBillingResult
        every { mockProductDetailsResult.productDetailsList } returns emptyList()

        // Act
        val result = billingRepository.queryProducts()

        // Assert
        assertIs<QueryProductResult.ItemUnavailable>(result)
    }

    @Test
    fun testQueryProductsBillingUnavailable() = runTest {
        // Arrange
        val mockBillingResult: BillingResult = mockk()
        val mockProductDetailsResult: ProductDetailsResult = mockk()

        every { mockBillingResult.responseCode } returns BillingResponseCode.BILLING_UNAVAILABLE
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryProductDetails(any()) } returns mockProductDetailsResult
        every { mockProductDetailsResult.billingResult } returns mockBillingResult
        every { mockProductDetailsResult.productDetailsList } returns emptyList()

        // Act
        val result = billingRepository.queryProducts()

        // Assert
        assertIs<QueryProductResult.BillingUnavailable>(result)
    }

    @Test
    fun testStartPurchaseFlowOk() = runTest {
        // Arrange
        val mockProductBillingResult: BillingResult = mockk()
        val mockBillingResult: BillingResult = mockk()
        val transactionId = "MOCK22"
        val productId = "TEST"
        val mockProductDetailsResult: ProductDetailsResult = mockk()
        val mockProductDetails: ProductDetails = mockk(relaxed = true)
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        every { mockBillingClient.launchBillingFlow(mockActivity, any()) } returns mockBillingResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)
        coEvery { mockBillingClient.queryProductDetails(any()) } returns mockProductDetailsResult
        every { mockProductDetailsResult.billingResult } returns mockProductBillingResult
        every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
        every { mockProductBillingResult.responseCode } returns BillingResponseCode.OK

        // Act
        val result = billingRepository.startPurchaseFlow(productId, transactionId)

        // Assert
        assertIs<PurchaseFlowResult.Ok>(result)
    }

    @Test
    fun testStartPurchaseFlowBillingUnavailable() = runTest {
        // Arrange
        val mockProductBillingResult: BillingResult = mockk()
        val mockBillingResult: BillingResult = mockk()
        val transactionId = "MOCK22"
        val productId = "TEST"
        val mockProductDetailsResult: ProductDetailsResult = mockk()
        val mockProductDetails: ProductDetails = mockk(relaxed = true)
        every { mockBillingResult.responseCode } returns BillingResponseCode.BILLING_UNAVAILABLE
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        every { mockBillingClient.launchBillingFlow(mockActivity, any()) } returns mockBillingResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)
        coEvery { mockBillingClient.queryProductDetails(any()) } returns mockProductDetailsResult
        every { mockProductDetailsResult.billingResult } returns mockProductBillingResult
        every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
        every { mockProductBillingResult.responseCode } returns BillingResponseCode.OK

        // Act
        val result = billingRepository.startPurchaseFlow(productId, transactionId)

        // Assert
        assertIs<PurchaseFlowResult.BillingUnavailable>(result)
    }

    @Test
    fun testQueryPurchasesFoundPurchases() = runTest {
        // Arrange
        val productId = "TEST"
        val token = "TOKEN"
        val expectedPurchase = BillingPurchase(productId = productId, token = token)
        val mockResult: PurchasesResult = mockk()
        val mockPurchase: Purchase = mockk()
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.purchasesList } returns listOf(mockPurchase)
        every { mockPurchase.toBillingPurchase() } returns expectedPurchase
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryPurchasesAsync(any<QueryPurchasesParams>()) } returns
            mockResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)

        // Act
        val result = billingRepository.queryPurchases()

        // Assert
        assertIs<QueryPurchasesResult.PurchaseFound>(result)
        assertEquals(expectedPurchase, result.purchase)
    }

    @Test
    fun testQueryPurchasesNoPurchaseFound() = runTest {
        // Arrange
        val mockResult: PurchasesResult = mockk()
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.purchasesList } returns emptyList()
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryPurchasesAsync(any<QueryPurchasesParams>()) } returns
            mockResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)

        // Act
        val result = billingRepository.queryPurchases()

        // Assert
        assertIs<QueryPurchasesResult.NoPurchasesFound>(result)
    }

    @Test
    fun testQueryPurchasesError() = runTest {
        // Arrange
        val responseCode = BillingResponseCode.ITEM_UNAVAILABLE
        val message = "ERROR"
        val expectedError = BillingException(responseCode, message)
        val mockResult: PurchasesResult = mockk()
        every { mockResult.billingResult.responseCode } returns responseCode
        every { mockResult.billingResult.debugMessage } returns message
        every { mockResult.purchasesList } returns emptyList()
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryPurchasesAsync(any<QueryPurchasesParams>()) } returns
            mockResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)

        // Act
        val result = billingRepository.queryPurchases()

        // Assert
        assertIs<QueryPurchasesResult.Error>(result)
        assertEquals(
            expectedError.toBillingResult().responseCode,
            result.exception.toBillingResult().responseCode
        )
        assertEquals(expectedError.message, result.exception.message)
    }

    @Test
    fun testPurchaseEventPurchaseComplete() = runTest {
        // Arrange
        val expectedPurchase: BillingPurchase = mockk()
        val mockPurchase: Purchase = mockk()
        val mockPurchaseList = listOf(mockPurchase)
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockPurchase.toBillingPurchase() } returns expectedPurchase

        // Act, Assert
        billingRepository.purchaseEvents.test {
            listenerSlot.captured.onPurchasesUpdated(mockBillingResult, mockPurchaseList)
            val result = awaitItem()
            assertIs<PurchaseEvent.PurchaseCompleted>(result)
            assertLists(listOf(expectedPurchase), result.purchases)
        }
    }

    @Test
    fun testPurchaseEventUserCanceled() = runTest {
        // Arrange
        val mockBillingResult: BillingResult = mockk()
        val mockResponseCode: Int = BillingResponseCode.USER_CANCELED
        every { mockBillingResult.responseCode } returns mockResponseCode

        // Act, Assert
        billingRepository.purchaseEvents.test {
            listenerSlot.captured.onPurchasesUpdated(mockBillingResult, null)
            val result = awaitItem()
            assertIs<PurchaseEvent.UserCanceled>(result)
        }
    }

    @Test
    fun testPurchaseEventError() = runTest {
        // Arrange
        val mockDebugMessage = "ERROR"
        val mockBillingResult: BillingResult = mockk()
        val mockResponseCode: Int = BillingResponseCode.ERROR
        val expectedError: BillingException =
            BillingException(responseCode = mockResponseCode, message = mockDebugMessage)
        every { mockBillingResult.responseCode } returns mockResponseCode
        every { mockBillingResult.debugMessage } returns mockDebugMessage

        // Act, Assert
        billingRepository.purchaseEvents.test {
            listenerSlot.captured.onPurchasesUpdated(mockBillingResult, null)
            val result = awaitItem()
            assertIs<PurchaseEvent.Error>(result)
            assertEquals(expectedError.message, result.exception.message)
        }
    }

    companion object {
        private const val BILLING_CLIENT_CLASS = "com.android.billingclient.api.BillingClient"
        private const val BILLING_CLIENT_KOTLIN_CLASS =
            "com.android.billingclient.api.BillingClientKotlinKt"
        private const val BILLING_FLOW_PARAMS = "com.android.billingclient.api.BillingFlowParams"
        private const val PURCHASE_EXTENSION =
            "net.mullvad.mullvadvpn.lib.billing.extension.PurchaseExtensionKt"
    }
}
