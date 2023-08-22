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
    fun testQueryProducts() = runTest {
        // Arrange
        val mockBillingResult: BillingResult = mockk()
        val expectedBillingResponseCode = BillingResponseCode.OK
        val expected =
            ProductDetailsResult(
                billingResult = mockBillingResult,
                productDetailsList = listOf(mockk()),
            )
        every { mockBillingResult.responseCode } returns expectedBillingResponseCode
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryProductDetails(any()) } returns expected

        // Act
        val result = billingRepository.queryProducts()

        // Assert
        assertEquals(expected.billingResult.responseCode, result.billingResult.responseCode)
        assertLists(
            expected.productDetailsList ?: emptyList(),
            result.productDetailsList ?: emptyList()
        )
    }

    @Test
    fun testStartPurchaseFlow() = runTest {
        // Arrange
        val expectedBillingResponseCode = BillingResponseCode.OK
        val expectedResult: BillingResult = mockk()
        val mockTransactionId = "MOCK"
        val mockProductDetails: ProductDetails = mockk(relaxed = true)
        every { expectedResult.responseCode } returns expectedBillingResponseCode
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        every { mockBillingClient.launchBillingFlow(mockActivity, any()) } returns expectedResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)

        // Act
        val result = billingRepository.startPurchaseFlow(mockProductDetails, mockTransactionId)

        // Assert
        assertEquals(expectedResult.responseCode, result.responseCode)
    }

    @Test
    fun testQueryPurchases() = runTest {
        // Arrange
        val expectedBillingResponseCode = BillingResponseCode.OK
        val expectedResult: PurchasesResult = mockk()
        every { expectedResult.billingResult.responseCode } returns expectedBillingResponseCode
        every { expectedResult.purchasesList } returns listOf(mockk())
        every { mockBillingClient.isReady } returns true
        every { mockBillingClient.connectionState } returns BillingClient.ConnectionState.CONNECTED
        coEvery { mockBillingClient.queryPurchasesAsync(any<QueryPurchasesParams>()) } returns
            expectedResult
        every { BillingFlowParams.newBuilder() } returns mockk(relaxed = true)

        // Act
        val result = billingRepository.queryPurchases()

        // Assert
        assertEquals(expectedResult.billingResult.responseCode, result.billingResult.responseCode)
        assertLists(expectedResult.purchasesList, result.purchasesList)
    }

    @Test
    fun testPurchaseEventPurchaseComplete() = runTest {
        // Arrange
        val mockPurchaseList = listOf(mockk<Purchase>())
        val mockBillingResult: BillingResult = mockk()
        val mockResponseCode: Int = BillingResponseCode.OK
        every { mockBillingResult.responseCode } returns mockResponseCode

        // Act, Assert
        billingRepository.purchaseEvents.test {
            listenerSlot.captured.onPurchasesUpdated(mockBillingResult, mockPurchaseList)
            val result = awaitItem()
            assertIs<PurchaseEvent.PurchaseCompleted>(result)
            assertEquals(mockPurchaseList, result.purchases)
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
        val mockBillingResult: BillingResult = mockk()
        val mockResponseCode: Int = BillingResponseCode.ERROR
        every { mockBillingResult.responseCode } returns mockResponseCode

        // Act, Assert
        billingRepository.purchaseEvents.test {
            listenerSlot.captured.onPurchasesUpdated(mockBillingResult, null)
            val result = awaitItem()
            assertIs<PurchaseEvent.Error>(result)
            assertEquals(mockBillingResult, result.result)
        }
    }

    companion object {
        private const val BILLING_CLIENT_CLASS = "com.android.billingclient.api.BillingClient"
        private const val BILLING_CLIENT_KOTLIN_CLASS =
            "com.android.billingclient.api.BillingClientKotlinKt"
        private const val BILLING_FLOW_PARAMS = "com.android.billingclient.api.BillingFlowParams"
    }
}
