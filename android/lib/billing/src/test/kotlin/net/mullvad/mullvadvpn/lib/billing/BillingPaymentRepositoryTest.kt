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
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class BillingPaymentRepositoryTest {

    private val mockBillingRepository: BillingRepository = mockk()
    private val mockPlayPurchaseRepository: PlayPurchaseRepository = mockk()

    private val purchaseEventFlow = MutableSharedFlow<PurchaseEvent>(extraBufferCapacity = 1)

    private lateinit var paymentRepository: BillingPaymentRepository

    @BeforeEach
    fun setup() {
        mockkStatic(PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT)

        every { mockBillingRepository.purchaseEvents } returns purchaseEventFlow

        paymentRepository =
            BillingPaymentRepository(
                billingRepository = mockBillingRepository,
                playPurchaseRepository = mockPlayPurchaseRepository
            )
    }

    @Test
    fun `given billingResult OK queryPaymentAvailability should return available products`() =
        runTest {
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
    fun `given billingResult OK but no products queryPaymentAvailability should return NoProductsFound`() =
        runTest {
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
                assertIs<PaymentAvailability.NoProductsFound>(result)
                awaitComplete()
            }
        }

    @Test
    fun `given billingResult is unavailable queryPaymentAvailability should return BillingUnavailable`() =
        runTest {
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
    fun `purchaseProduct should return FetchProductsError on billing unavailable`() = runTest {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockProductDetailsResult = mockk<ProductDetailsResult>()
        every { mockProductDetailsResult.billingResult.responseCode } returns
            BillingResponseCode.BILLING_UNAVAILABLE
        every { mockProductDetailsResult.billingResult.debugMessage } returns "ERROR"
        coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
            mockProductDetailsResult

        // Act, Assert
        paymentRepository.purchaseProduct(mockProductId, mockk()).test {
            assertIs<PurchaseResult.FetchingProducts>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.Error.FetchProductsError>(result)
            awaitComplete()
        }
    }

    @Test
    fun `purchaseProduct should return NoProductFound when billingRepository does not find any products`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockProductDetailsResult = mockk<ProductDetailsResult>()
            every { mockProductDetailsResult.billingResult.responseCode } returns
                BillingResponseCode.OK
            every { mockProductDetailsResult.productDetailsList } returns emptyList()
            coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
                mockProductDetailsResult

            // Act, Assert
            paymentRepository.purchaseProduct(mockProductId, mockk()).test {
                assertIs<PurchaseResult.FetchingProducts>(awaitItem())
                val result = awaitItem()
                assertIs<PurchaseResult.Error.NoProductFound>(result)
                awaitComplete()
            }
        }

    @Test
    fun `purchaseProduct should return TransactionIdError on PlayPurchaseInitError from PlayPurchaseRepository`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockProductDetailsResult = mockk<ProductDetailsResult>()
            val mockProductDetails: ProductDetails = mockk()
            every { mockProductDetails.productId } returns mockProductId.value
            every { mockProductDetailsResult.billingResult.responseCode } returns
                BillingResponseCode.OK
            every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
            coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
                mockProductDetailsResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayPurchaseInitResult.Error(PlayPurchaseInitError.OtherError)

            // Act, Assert
            paymentRepository.purchaseProduct(mockProductId, mockk()).test {
                assertIs<PurchaseResult.FetchingProducts>(awaitItem())
                assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
                val result = awaitItem()
                assertIs<PurchaseResult.Error.TransactionIdError>(result)
                awaitComplete()
            }
        }

    @Test
    fun `purchaseProduct should return BillingError on billing unavailable from startPurchaseFlow`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockProductDetailsResult = mockk<ProductDetailsResult>()
            val mockProductDetails: ProductDetails = mockk()
            every { mockProductDetails.productId } returns mockProductId.value
            every { mockProductDetailsResult.billingResult.responseCode } returns
                BillingResponseCode.OK
            every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
            coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
                mockProductDetailsResult
            val mockBillingResult: BillingResult = mockk()
            every { mockBillingResult.responseCode } returns BillingResponseCode.BILLING_UNAVAILABLE
            every { mockBillingResult.debugMessage } returns "Mock error"
            coEvery {
                mockBillingRepository.startPurchaseFlow(
                    productDetails = any(),
                    obfuscatedId = any(),
                    activityProvider = any()
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayPurchaseInitResult.Ok("MOCK")

            // Act, Assert
            paymentRepository.purchaseProduct(mockProductId, mockk()).test {
                // Purchase started
                assertIs<PurchaseResult.FetchingProducts>(awaitItem())
                assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
                val result = awaitItem()
                assertIs<PurchaseResult.Error.BillingError>(result)
                awaitComplete()
            }
        }

    @Test
    fun `cancel during purchaseProduct should return in cancelled event`() = runTest {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockProductDetailsResult = mockk<ProductDetailsResult>()
        val mockProductDetails: ProductDetails = mockk()
        every { mockProductDetails.productId } returns mockProductId.value
        every { mockProductDetailsResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
        coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
            mockProductDetailsResult
        val mockObfuscatedId = "MOCK-ID"
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productDetails = any(),
                obfuscatedId = mockObfuscatedId,
                activityProvider = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
            PlayPurchaseInitResult.Ok(mockObfuscatedId)

        // Act, Assert
        paymentRepository.purchaseProduct(mockProductId, mockk()).test {
            assertIs<PurchaseResult.FetchingProducts>(awaitItem())
            assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.UserCanceled)
            val result = awaitItem()
            assertIs<PurchaseResult.Completed.Cancelled>(result)
            awaitComplete()
        }
    }

    @Test
    fun `purchaseProduct should emit VerificationError on verify error from playPurchase`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockProductDetailsResult = mockk<ProductDetailsResult>()
            val mockProductDetails: ProductDetails = mockk()
            every { mockProductDetails.productId } returns mockProductId.value
            every { mockProductDetailsResult.billingResult.responseCode } returns
                BillingResponseCode.OK
            every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
            coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
                mockProductDetailsResult
            val mockPurchaseToken = "TOKEN"
            val mockBillingPurchase: Purchase = mockk()
            val mockBillingResult: BillingResult = mockk()
            every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
            every { mockBillingResult.responseCode } returns BillingResponseCode.OK
            every { mockBillingPurchase.products } returns listOf(mockProductId.value)
            every { mockBillingPurchase.purchaseToken } returns mockPurchaseToken
            coEvery {
                mockBillingRepository.startPurchaseFlow(
                    productDetails = any(),
                    obfuscatedId = any(),
                    activityProvider = any()
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayPurchaseInitResult.Ok("MOCK-ID")
            coEvery { mockPlayPurchaseRepository.verifyPlayPurchase(any()) } returns
                PlayPurchaseVerifyResult.Error(PlayPurchaseVerifyError.OtherError)

            // Act, Assert
            paymentRepository.purchaseProduct(mockProductId, mockk()).test {
                assertIs<PurchaseResult.FetchingProducts>(awaitItem())
                assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
                assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
                purchaseEventFlow.tryEmit(PurchaseEvent.Completed(listOf(mockBillingPurchase)))
                assertIs<PurchaseResult.VerificationStarted>(awaitItem())
                val result = awaitItem()
                assertIs<PurchaseResult.Error.VerificationError>(result)
                awaitComplete()
            }
        }

    @Test
    fun `purchaseProduct success should emit all events leading up to success`() = runTest {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockProductDetailsResult = mockk<ProductDetailsResult>()
        val mockProductDetails: ProductDetails = mockk()
        every { mockProductDetails.productId } returns mockProductId.value
        every { mockProductDetailsResult.billingResult.responseCode } returns BillingResponseCode.OK
        every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
        coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
            mockProductDetailsResult
        val mockPurchaseToken = "TOKEN"
        val mockBillingPurchase: Purchase = mockk()
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingPurchase.products } returns listOf(mockProductId.value)
        every { mockBillingPurchase.purchaseToken } returns mockPurchaseToken
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productDetails = any(),
                obfuscatedId = any(),
                activityProvider = any()
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
            PlayPurchaseInitResult.Ok("MOCK")
        coEvery { mockPlayPurchaseRepository.verifyPlayPurchase(any()) } returns
            PlayPurchaseVerifyResult.Ok

        // Act, Assert
        paymentRepository.purchaseProduct(mockProductId, mockk()).test {
            assertIs<PurchaseResult.FetchingProducts>(awaitItem())
            assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
            assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
            purchaseEventFlow.tryEmit(PurchaseEvent.Completed(listOf(mockBillingPurchase)))
            assertIs<PurchaseResult.VerificationStarted>(awaitItem())
            val result = awaitItem()
            assertIs<PurchaseResult.Completed.Success>(result)
            awaitComplete()
        }
    }

    @Test
    fun `purchaseProduct where purchase gets stuck in pending should complete with pending`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockProductDetailsResult = mockk<ProductDetailsResult>()
            val mockProductDetails: ProductDetails = mockk()
            every { mockProductDetails.productId } returns mockProductId.value
            every { mockProductDetailsResult.billingResult.responseCode } returns
                BillingResponseCode.OK
            every { mockProductDetailsResult.productDetailsList } returns listOf(mockProductDetails)
            coEvery { mockBillingRepository.queryProducts(listOf(mockProductId.value)) } returns
                mockProductDetailsResult
            val mockBillingPurchase: Purchase = mockk()
            val mockBillingResult: BillingResult = mockk()
            every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PENDING
            every { mockBillingResult.responseCode } returns BillingResponseCode.OK
            coEvery {
                mockBillingRepository.startPurchaseFlow(
                    productDetails = any(),
                    obfuscatedId = any(),
                    activityProvider = any()
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayPurchaseInitResult.Ok("MOCK")

            // Act, Assert
            paymentRepository.purchaseProduct(mockProductId, mockk()).test {
                assertIs<PurchaseResult.FetchingProducts>(awaitItem())
                assertIs<PurchaseResult.FetchingObfuscationId>(awaitItem())
                assertIs<PurchaseResult.BillingFlowStarted>(awaitItem())
                purchaseEventFlow.tryEmit(PurchaseEvent.Completed(listOf(mockBillingPurchase)))
                val result = awaitItem()
                assertIs<PurchaseResult.Completed.Pending>(result)
                awaitComplete()
            }
        }

    companion object {
        private const val PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT =
            "net.mullvad.mullvadvpn.lib.billing.extension.ProductDetailsToPaymentProductKt"
    }
}
