package net.mullvad.mullvadvpn.lib.payment

import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runBlockingTest
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.billing.BillingRepository
import net.mullvad.mullvadvpn.lib.billing.model.BillingProduct
import net.mullvad.mullvadvpn.lib.billing.model.BillingPurchase
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseEvent
import net.mullvad.mullvadvpn.lib.billing.model.PurchaseFlowResult
import net.mullvad.mullvadvpn.lib.billing.model.QueryProductResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class PaymentRepositoryTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockBillingRepository: BillingRepository = mockk()

    private val purchaseEventFlow =
        MutableSharedFlow<PurchaseEvent>(replay = 1, extraBufferCapacity = 1)

    private lateinit var paymentRepository: PaymentRepository

    @Before
    fun setUp() {
        every { mockBillingRepository.purchaseEvents } returns purchaseEventFlow
    }

    @After fun tearDown() {}

    @Test
    fun testQueryAvailablePaymentShowWebPayment() = runTest {
        // Arrange
        paymentRepository =
            PaymentRepository(billingRepository = mockBillingRepository, showWebPayment = true)
        coEvery { mockBillingRepository.queryProducts(any()) } returns
            QueryProductResult.ItemUnavailable

        // Act
        val result = paymentRepository.queryAvailablePaymentTypes()

        // Assert
        assert(result.webPaymentAvailable)
    }

    @Test
    fun testQueryAvailablePaymentHideWebPayment() = runTest {
        // Arrange
        paymentRepository =
            PaymentRepository(billingRepository = mockBillingRepository, showWebPayment = false)
        coEvery { mockBillingRepository.queryProducts(any()) } returns
            QueryProductResult.ItemUnavailable

        // Act
        val result = paymentRepository.queryAvailablePaymentTypes()

        // Assert
        assertFalse(result.webPaymentAvailable)
    }

    @Test
    fun testQueryAvailablePaymentProductsAvailable() = runTest {
        // Arrange
        paymentRepository =
            PaymentRepository(billingRepository = mockBillingRepository, showWebPayment = false)
        val mockProduct: BillingProduct = mockk()
        coEvery { mockBillingRepository.queryProducts(any()) } returns
            QueryProductResult.Ok(products = listOf(mockProduct))

        // Act
        val result = paymentRepository.queryAvailablePaymentTypes().billingPaymentAvailability

        // Assert
        assertIs<BillingPaymentAvailability.ProductsAvailable>(result)
        assertEquals(mockProduct, result.products.first())
    }

    @Test
    fun testQueryAvailablePaymentProductsUnavailable() = runTest {
        // Arrange
        paymentRepository =
            PaymentRepository(billingRepository = mockBillingRepository, showWebPayment = false)
        coEvery { mockBillingRepository.queryProducts(any()) } returns
            QueryProductResult.ItemUnavailable

        // Act
        val result = paymentRepository.queryAvailablePaymentTypes().billingPaymentAvailability

        // Assert
        assertIs<BillingPaymentAvailability.ProductsUnavailable>(result)
    }

    @Test
    fun testQueryAvailablePaymentBillingUnavailableError() = runTest {
        // Arrange
        paymentRepository =
            PaymentRepository(billingRepository = mockBillingRepository, showWebPayment = false)
        coEvery { mockBillingRepository.queryProducts(any()) } returns
            QueryProductResult.BillingUnavailable

        // Act
        val result = paymentRepository.queryAvailablePaymentTypes().billingPaymentAvailability

        // Assert
        assertIs<BillingPaymentAvailability.Error.BillingUnavailable>(result)
    }
}
