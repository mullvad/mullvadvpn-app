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
import net.mullvad.mullvadvpn.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class BillingPaymentRepositoryTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockBillingRepository: BillingRepository = mockk()
    private val mockPlayPurchaseRepository: PlayPurchaseRepository = mockk()

    private val purchaseEventFlow =
        MutableSharedFlow<PurchaseEvent>(replay = 1, extraBufferCapacity = 1)

    private lateinit var paymentRepository: BillingPaymentRepository

    @Before
    fun setUp() {
        mockkStatic(PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT)

        every { mockBillingRepository.purchaseEvents } returns purchaseEventFlow

        paymentRepository =
            BillingPaymentRepository(
                billingRepository = mockBillingRepository,
                playPurchaseRepository = mockPlayPurchaseRepository
            )
    }

    @After fun tearDown() {}

    @Test
    fun testQueryAvailablePaymentProductsAvailable() = runTest {
        // Arrange
        val expectedProduct: PaymentProduct = mockk()
        val mockProduct: ProductDetails = mockk()
        val mockResult: ProductDetailsResult = mockk()
        coEvery { mockBillingRepository.queryPurchases() } returns mockk(relaxed = true)
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult
        every { mockProduct.toPaymentProduct(any()) } returns expectedProduct
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.productDetailsList } returns listOf(mockProduct)

        // Act, Assert
        paymentRepository.queryPaymentAvailability().test {
            // Loading
            awaitItem()
            val result = awaitItem()
            assertIs<PaymentAvailability.ProductsAvailable>(result)
            assertEquals(expectedProduct, result.products.first())
            awaitComplete()
        }
    }

    @Test
    fun testQueryAvailablePaymentProductsUnavailable() = runTest {
        // Arrange
        val mockResult: ProductDetailsResult = mockk()
        every { mockResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockResult.productDetailsList } returns emptyList()
        coEvery { mockBillingRepository.queryPurchases() } returns mockk(relaxed = true)
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult

        // Act, Assert
        paymentRepository.queryPaymentAvailability().test {
            // Loading
            awaitItem()
            val result = awaitItem()
            assertIs<PaymentAvailability.ProductsUnavailable>(result)
            awaitComplete()
        }
    }

    @Test
    fun testQueryAvailablePaymentBillingUnavailableError() = runTest {
        // Arrange
        val mockResult: ProductDetailsResult = mockk()
        every { mockResult.billingResult.responseCode } returns
            BillingResponseCode.BILLING_UNAVAILABLE
        coEvery { mockBillingRepository.queryPurchases() } returns mockk(relaxed = true)
        coEvery { mockBillingRepository.queryProducts(any()) } returns mockResult

        // Act, Assert
        paymentRepository.queryPaymentAvailability().test {
            // Loading
            awaitItem()
            val result = awaitItem()
            assertIs<PaymentAvailability.Error.BillingUnavailable>(result)
            awaitComplete()
        }
    }

    @Test
    fun testPurchaseBillingProductStartPurchaseTransactionIdError() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Error(PlayPurchaseInitError.OtherError)

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.Error.TransactionIdError>(result)
            awaitComplete()
        }
    }

    @Test
    fun testPurchaseBillingProductStartPurchaseFlowBillingError() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.BILLING_UNAVAILABLE
        every { mockBillingResult.debugMessage } returns "Mock error"
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Ok("MOCK")

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.Error.BillingError>(result)
            awaitComplete()
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
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Ok("MOCK")

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            // Billing flow started
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.UserCanceled)
            val result = awaitItem()
            assertIs<PurchaseResult.PurchaseCancelled>(result)
            awaitComplete()
        }
    }

    @Test
    fun testPurchaseBillingProductVerificationError() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockPurchaseTokem = "TOKEN"
        val mockBillingPurchase: Purchase = mockk()
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingPurchase.products } returns listOf(mockProductId)
        every { mockBillingPurchase.purchaseToken } returns mockPurchaseTokem
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Ok("MOCK")
        coEvery { mockPlayPurchaseRepository.purchaseVerification(any()) } returns
            PlayPurchaseVerifyResult.Error(PlayPurchaseVerifyError.OtherError)

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            // Billing flow started
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.PurchaseCompleted(listOf(mockBillingPurchase)))
            // Verification started
            assertIs<PurchaseResult.VerificationStarted>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.Error.VerificationError>(result)
            awaitComplete()
        }
    }

    @Test
    fun testPurchaseBillingProductPurchaseCompleted() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockPurchaseTokem = "TOKEN"
        val mockBillingPurchase: Purchase = mockk()
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingPurchase.products } returns listOf(mockProductId)
        every { mockBillingPurchase.purchaseToken } returns mockPurchaseTokem
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Ok("MOCK")
        coEvery { mockPlayPurchaseRepository.purchaseVerification(any()) } returns
            PlayPurchaseVerifyResult.Ok

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            // Billing flow started
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.PurchaseCompleted(listOf(mockBillingPurchase)))
            // Verification started
            assertIs<PurchaseResult.VerificationStarted>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.PurchaseCompleted>(result)
            awaitComplete()
        }
    }

    @Test
    fun testPurchaseBillingProductPurchasePending() = runTest {
        // Arrange
        val mockProductId = "MOCK"
        val mockBillingPurchase: Purchase = mockk()
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PENDING
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productId = mockProductId,
                transactionId = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.purchaseInitialisation() } returns
            PlayPurchaseInitResult.Ok("MOCK")

        // Act, Assert
        paymentRepository.purchaseBillingProduct(mockProductId).test {
            // Purchase started
            assertIs<PurchaseResult.PurchaseStarted>(awaitItem())
            // Billing flow started
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.PurchaseCompleted(listOf(mockBillingPurchase)))
            // Purchase pending
            val result = awaitItem()
            assertIs<PurchaseResult.PurchasePending>(result)
            awaitComplete()
        }
    }

    companion object {
        private const val PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT =
            "net.mullvad.mullvadvpn.lib.billing.extension.ProductDetailsToPaymentProductKt"
    }
}
