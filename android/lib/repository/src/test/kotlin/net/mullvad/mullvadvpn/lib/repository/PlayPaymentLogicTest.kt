package net.mullvad.mullvadvpn.lib.repository

import app.cash.turbine.test
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import org.junit.jupiter.api.Test

class PlayPaymentLogicTest {

    private val mockPaymentRepository: PaymentRepository = mockk(relaxed = true)

    private val playPaymentLogic = PlayPaymentLogic(mockPaymentRepository)

    @Test
    fun `queryPaymentAvailability call should result in updated paymentAvailability`() = runTest {
        // Arrange
        val productsUnavailable = PaymentAvailability.ProductsUnavailable
        val paymentRepositoryQueryPaymentAvailabilityFlow = flow { emit(productsUnavailable) }
        every { mockPaymentRepository.queryPaymentAvailability() } returns
            paymentRepositoryQueryPaymentAvailabilityFlow

        // Act, Assert
        playPaymentLogic.paymentAvailability.test {
            assertNull(awaitItem())
            playPaymentLogic.queryPaymentAvailability()
            assertEquals(productsUnavailable, awaitItem())
        }
    }

    @Test
    fun `purchaseProduct call should result in updated purchaseResult`() = runTest {
        // Arrange
        val fetchingProducts = PurchaseResult.FetchingProducts
        val productId = ProductId("productId")
        val paymentRepositoryPurchaseResultFlow = flow { emit(fetchingProducts) }
        every { mockPaymentRepository.purchaseProduct(any(), any()) } returns
            paymentRepositoryPurchaseResultFlow

        // Act, Assert
        playPaymentLogic.purchaseResult.test {
            assertNull(awaitItem())
            playPaymentLogic.purchaseProduct(productId, mockk())
            assertEquals(fetchingProducts, awaitItem())
        }
    }

    @Test
    fun `purchaseProduct call should invoke purchaseProduct on repository`() = runTest {
        // Arrange
        val productId = ProductId("productId")

        // Act
        playPaymentLogic.purchaseProduct(productId, mockk())

        // Assert
        coVerify { mockPaymentRepository.purchaseProduct(productId, any()) }
    }

    @Test
    fun `queryPaymentAvailability should invoke queryPaymentAvailability on repository`() =
        runTest {
            // Act
            playPaymentLogic.queryPaymentAvailability()

            // Assert
            coVerify { mockPaymentRepository.queryPaymentAvailability() }
        }

    @Test
    fun `resetPurchaseResult call should result in purchaseResult null`() = runTest {
        // Arrange
        val completedSuccess = PurchaseResult.Completed.Success(ProductId("one_month"))
        val productId = ProductId("productId")
        val paymentRepositoryPurchaseResultFlow = flow { emit(completedSuccess) }
        every { mockPaymentRepository.purchaseProduct(any(), any()) } returns
            paymentRepositoryPurchaseResultFlow

        // Act, Assert
        playPaymentLogic.purchaseResult.test {
            assertNull(awaitItem())
            playPaymentLogic.purchaseProduct(productId, mockk())
            assertEquals(completedSuccess, awaitItem())
            playPaymentLogic.resetPurchaseResult()
            assertNull(awaitItem())
        }
    }

    @Test
    fun `verifyPurchases call should invoke verifyPurchases on repository`() = runTest {
        // Act
        playPaymentLogic.verifyPurchases()

        // Assert
        coVerify { mockPaymentRepository.verifyPurchases() }
    }
}
