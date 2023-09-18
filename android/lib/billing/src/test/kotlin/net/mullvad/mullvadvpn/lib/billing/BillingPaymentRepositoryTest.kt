package net.mullvad.mullvadvpn.lib.billing

import app.cash.turbine.test
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetailsResult
import com.android.billingclient.api.Purchase
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.billing.extension.toPaymentProduct
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class BillingPaymentRepositoryTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockBillingRepository: BillingRepository = mockk()

    private val purchaseEventFlow =
        MutableSharedFlow<PurchaseEvent>(replay = 1, extraBufferCapacity = 1)

    private lateinit var paymentRepository: BillingPaymentRepository

    @Before
    fun setUp() {
        mockkStatic(PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT)

        every { mockBillingRepository.purchaseEvents } returns purchaseEventFlow

        paymentRepository = BillingPaymentRepository(billingRepository = mockBillingRepository)
    }

    @After fun tearDown() {}

    @Test
    fun testQueryAvailablePaymentProductsAvailable() = runTest {
        // Arrange
        val expectedProduct: PaymentProduct = mockk()
        val mockProduct: ProductDetails = mockk()
        val mockResult: ProductDetailsResult = mockk()
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult
        every { mockProduct.toPaymentProduct() } returns expectedProduct
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.productDetailsList } returns listOf(mockProduct)

        // Act
        val result = paymentRepository.queryPaymentAvailability()

        // Assert
        assertIs<PaymentAvailability.ProductsAvailable>(result)
        assertEquals(expectedProduct, result.products.first())
    }

    @Test
    fun testQueryAvailablePaymentProductsUnavailable() = runTest {
        // Arrange
        val mockResult: ProductDetailsResult = mockk()
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.productDetailsList } returns emptyList()
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult

        // Act
        val result = paymentRepository.queryPaymentAvailability()

        // Assert
        assertIs<PaymentAvailability.ProductsUnavailable>(result)
    }

    @Test
    fun testQueryAvailablePaymentBillingUnavailableError() = runTest {
        // Arrange
        val mockResult: ProductDetailsResult = mockk()
        every { mockResult.billingResult.responseCode } returns
            BillingResponseCode.BILLING_UNAVAILABLE
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult

        // Act
        val result = paymentRepository.queryPaymentAvailability()

        // Assert
        assertIs<PaymentAvailability.Error.BillingUnavailable>(result)
    }

    @Test
    fun testPurchaseBillingProductStartPurchaseFlowError() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.BILLING_UNAVAILABLE
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult

        // Act, Assert
        paymentRepository.purchaseResult.test {
            paymentRepository.purchaseBillingProduct(mockProductId)
            val result = awaitItem()
            assertIs<PurchaseResult.PurchaseError>(result)
        }
    }

    @Test
    fun testPurchaseBillingProductPurchaseCanceled() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult

        // Act, Assert
        paymentRepository.purchaseResult.test {
            paymentRepository.purchaseBillingProduct(mockProductId)
            purchaseEventFlow.tryEmit(PurchaseEvent.UserCanceled)
            val result = awaitItem()
            assertIs<PurchaseResult.PurchaseCancelled>(result)
        }
    }

    @Test
    fun testPurchaseBillingProductPurchaseCompleted() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockBillingPurchase: Purchase = mockk()
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult

        // Act, Assert
        paymentRepository.purchaseResult.test {
            paymentRepository.purchaseBillingProduct(mockProductId)
            purchaseEventFlow.tryEmit(PurchaseEvent.PurchaseCompleted(listOf(mockBillingPurchase)))
            val result = awaitItem()
            assertIs<PurchaseResult.PurchaseCompleted>(result)
        }
    }

    companion object {
        private const val PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT =
            "net.mullvad.mullvadvpn.lib.billing.extension.ProductDetailsToPaymentProductKt"
    }
}
