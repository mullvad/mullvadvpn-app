package net.mullvad.mullvadvpn.usecase

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

class PlayPaymentUseCaseTest {

    private val mockPaymentRepository: PaymentRepository = mockk(relaxed = true)

    private val playPaymentUseCase = PlayPaymentUseCase(mockPaymentRepository)

    @Test
    fun `queryPaymentAvailability call should result in updated paymentAvailability`() = runTest {
        // Arrange
        val productsUnavailable = PaymentAvailability.ProductsUnavailable
        val paymentRepositoryQueryPaymentAvailabilityFlow = flow { emit(productsUnavailable) }
        every { mockPaymentRepository.queryPaymentAvailability() } returns
            paymentRepositoryQueryPaymentAvailabilityFlow

        // Act, Assert
        playPaymentUseCase.paymentAvailability.test {
            assertNull(awaitItem())
            playPaymentUseCase.queryPaymentAvailability()
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
        playPaymentUseCase.purchaseResult.test {
            assertNull(awaitItem())
            playPaymentUseCase.purchaseProduct(productId, mockk())
            assertEquals(fetchingProducts, awaitItem())
        }
    }

    @Test
    fun `purchaseProduct call should invoke purchaseProduct on repository`() = runTest {
        // Arrange
        val productId = ProductId("productId")

        // Act
        playPaymentUseCase.purchaseProduct(productId, mockk())

        // Assert
        coVerify { mockPaymentRepository.purchaseProduct(productId, any()) }
    }

    @Test
    fun `queryPaymentAvailability should invoke queryPaymentAvailability on repository`() =
        runTest {
            // Act
            playPaymentUseCase.queryPaymentAvailability()

            // Assert
            coVerify { mockPaymentRepository.queryPaymentAvailability() }
        }

    @Test
    fun `on resetPurchaseResult call purchaseResult should emit null`() = runTest {
        // Arrange
        val completedSuccess = PurchaseResult.Completed.Success
        val productId = ProductId("productId")
        val paymentRepositoryPurchaseResultFlow = flow { emit(completedSuccess) }
        every { mockPaymentRepository.purchaseProduct(any(), any()) } returns
            paymentRepositoryPurchaseResultFlow

        // Act, Assert
        playPaymentUseCase.purchaseResult.test {
            assertNull(awaitItem())
            playPaymentUseCase.purchaseProduct(productId, mockk())
            assertEquals(completedSuccess, awaitItem())
            playPaymentUseCase.resetPurchaseResult()
            assertNull(awaitItem())
        }
    }

    @Test
    fun `verifyPurchases call should invoke verifyPurchases on repository`() = runTest {
        // Act
        playPaymentUseCase.verifyPurchases()

        // Assert
        coVerify { mockPaymentRepository.verifyPurchases() }
    }
}
