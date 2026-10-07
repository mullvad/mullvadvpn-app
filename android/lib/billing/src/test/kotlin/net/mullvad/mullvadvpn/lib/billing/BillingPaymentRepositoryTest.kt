package net.mullvad.mullvadvpn.lib.billing

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import com.android.billingclient.api.AccountIdentifiers
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.ProductDetailsResult
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesResult
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.billing.extension.toPaymentProduct
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.PlayExternalObfuscatedAccountId
import net.mullvad.mullvadvpn.lib.model.PlayPurchaseInitError
import net.mullvad.mullvadvpn.lib.model.PlayPurchaseVerifyError
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class BillingPaymentRepositoryTest {

    private val mockBillingRepository: BillingRepository = mockk()
    private val mockPlayPurchaseRepository: PlayPurchaseRepository = mockk()
    private val mockUserPreferencesRepository: UserPreferencesRepository = mockk()

    private val purchaseEventFlow = MutableSharedFlow<PurchaseEvent>(extraBufferCapacity = 1)

    private lateinit var paymentRepository: BillingPaymentRepository

    @BeforeEach
    fun setup() {
        mockkStatic(PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT)

        every { mockBillingRepository.purchaseEvents } returns purchaseEventFlow
        coEvery { mockUserPreferencesRepository.clearLatestSuccessfulPurchase() } just Runs

        paymentRepository =
            BillingPaymentRepository(
                billingRepository = mockBillingRepository,
                playPurchaseRepository = mockPlayPurchaseRepository,
                userPreferenceRepository = mockUserPreferencesRepository,
            )
    }

    @Test
    fun `queryPaymentAvailability should return available products when billing is OK`() = runTest {
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
    fun `queryPaymentAvailability should return NoProductsFound when billing is OK with no products `() =
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
    fun `queryPaymentAvailability should return BillingUnavailable when billing is Unavailable `() =
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
    fun `purchaseProduct should return FetchProductsError when billing is Unavailable`() = runTest {
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
                PlayPurchaseInitError.OtherError.left()

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
    fun `purchaseProduct should return TransactionIdError when PlayPurchasePaymentToken is empty`() =
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
                PlayExternalObfuscatedAccountId("").right()

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
                    activityProvider = any(),
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayExternalObfuscatedAccountId("MOCK").right()

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
        val mockObfuscatedId = PlayExternalObfuscatedAccountId("MOCK-ID")
        val mockBillingResult: BillingResult = mockk()
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productDetails = any(),
                obfuscatedId = mockObfuscatedId,
                activityProvider = any(),
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
            mockObfuscatedId.right()

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
            val mockAccountIdentifiers: AccountIdentifiers = mockk()
            every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
            every { mockBillingResult.responseCode } returns BillingResponseCode.OK
            every { mockBillingPurchase.products } returns listOf(mockProductId.value)
            every { mockBillingPurchase.purchaseToken } returns mockPurchaseToken
            every { mockBillingPurchase.accountIdentifiers } returns mockAccountIdentifiers
            every { mockAccountIdentifiers.obfuscatedAccountId } returns "Something"
            coEvery {
                mockBillingRepository.startPurchaseFlow(
                    productDetails = any(),
                    obfuscatedId = any(),
                    activityProvider = any(),
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayExternalObfuscatedAccountId("MOCK-ID").right()
            coEvery { mockPlayPurchaseRepository.verifyPlayPurchase(any()) } returns
                PlayPurchaseVerifyError.OtherError.left()

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
        val mockAccountIdentifiers: AccountIdentifiers = mockk()
        every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
        every { mockBillingResult.responseCode } returns BillingResponseCode.OK
        every { mockBillingPurchase.products } returns listOf(mockProductId.value)
        every { mockBillingPurchase.purchaseToken } returns mockPurchaseToken
        every { mockBillingPurchase.accountIdentifiers } returns mockAccountIdentifiers
        every { mockAccountIdentifiers.obfuscatedAccountId } returns "Something"
        coEvery {
            mockBillingRepository.startPurchaseFlow(
                productDetails = any(),
                obfuscatedId = any(),
                activityProvider = any(),
            )
        } returns mockBillingResult
        coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
            PlayExternalObfuscatedAccountId("MOCK").right()
        coEvery { mockPlayPurchaseRepository.verifyPlayPurchase(any()) } returns Unit.right()
        coEvery {
            mockUserPreferencesRepository.setLatestSuccessfulPurchase(mockPurchaseToken)
        } just Runs

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
            every { mockBillingPurchase.products } returns listOf("MOCK")
            every { mockBillingResult.responseCode } returns BillingResponseCode.OK
            coEvery {
                mockBillingRepository.startPurchaseFlow(
                    productDetails = any(),
                    obfuscatedId = any(),
                    activityProvider = any(),
                )
            } returns mockBillingResult
            coEvery { mockPlayPurchaseRepository.initializePlayPurchase() } returns
                PlayExternalObfuscatedAccountId("MOCK").right()

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

    @Test
    fun `when user preferences returns a successful purchase token should ignore any unverified purchase with the same token`() =
        runTest {
            // Arrange
            val mockProductId = ProductId("MOCK")
            val mockPurchaseToken = "TOKEN"
            val mockBillingPurchase: Purchase = mockk()
            val mockPurchasesResult: PurchasesResult = mockk()
            val mockProductDetailsResult: ProductDetailsResult = mockk()
            val mockBillingProduct: ProductDetails = mockk()
            every { mockBillingPurchase.purchaseState } returns Purchase.PurchaseState.PURCHASED
            every { mockBillingPurchase.products } returns listOf(mockProductId.value)
            every { mockBillingPurchase.purchaseToken } returns mockPurchaseToken
            every { mockBillingProduct.productId } returns mockProductId.value
            every { mockBillingProduct.oneTimePurchaseOfferDetails?.formattedPrice } returns "5.00€"
            every { mockPurchasesResult.billingResult } returns
                BillingResult.newBuilder().setResponseCode(BillingResponseCode.OK).build()
            every { mockProductDetailsResult.billingResult } returns
                BillingResult.newBuilder().setResponseCode(BillingResponseCode.OK).build()
            every { mockPurchasesResult.purchasesList } returns listOf(mockBillingPurchase)
            every { mockProductDetailsResult.productDetailsList } returns listOf(mockBillingProduct)
            coEvery {
                mockUserPreferencesRepository.latestSuccessfulPurchase()
            } returns mockPurchaseToken
            coEvery {
                mockBillingRepository.queryPurchases()
            } returns mockPurchasesResult
            coEvery {
                mockBillingRepository.queryProducts(any())
            } returns mockProductDetailsResult

            paymentRepository.queryPaymentAvailability().test {
                // Loading
                awaitItem()
                val result = awaitItem()
                assertIs<PaymentAvailability.ProductsAvailable>(result)
                assertEquals(mockProductId.value, result.products.first().productId.value)
                assertEquals(
                    false,
                    result.products.any { it.status == PaymentStatus.PURCHASED_UNVERIFIED },
                )
                awaitComplete()
            }
        }

    @Test
    fun `when query purchases returns empty list should clear latest successful purchase`() =
        runTest {
            // Arrange
            val mockPurchasesResult: PurchasesResult = mockk()
            every { mockPurchasesResult.billingResult } returns
                BillingResult.newBuilder().setResponseCode(BillingResponseCode.OK).build()
            every { mockPurchasesResult.purchasesList } returns emptyList()
            coEvery {
                mockBillingRepository.queryPurchases()
            } returns mockPurchasesResult
            coEvery {
                mockBillingRepository.queryProducts(any())
            } returns
                mockk<ProductDetailsResult>().also {
                    every { it.billingResult } returns
                        BillingResult.newBuilder().setResponseCode(BillingResponseCode.OK).build()
                    every { it.productDetailsList } returns emptyList()
                }

            // Act, Assert
            paymentRepository.queryPaymentAvailability().test {
                // Loading
                awaitItem()
                val result = awaitItem()
                assertIs<PaymentAvailability.NoProductsFound>(result)
                coVerify(exactly = 1) {
                    mockUserPreferencesRepository.clearLatestSuccessfulPurchase()
                }
                awaitComplete()
            }
        }

    companion object {
        private const val PRODUCT_DETAILS_TO_PAYMENT_PRODUCT_EXT =
            "net.mullvad.mullvadvpn.lib.billing.extension.ProductDetailsToPaymentProductKt"
    }
}
