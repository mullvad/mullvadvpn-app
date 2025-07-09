package net.mullvad.mullvadvpn.viewmodel

import android.app.Activity
import app.cash.turbine.test
import arrow.core.right
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class AddTimeViewModelTest {

    private val mockPaymentUseCase: PaymentUseCase = mockk()
    private val mockAccountRepository: AccountRepository = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    private val paymentAvailability = MutableStateFlow<PaymentAvailability?>(null)
    private val purchaseResult = MutableStateFlow<PurchaseResult?>(null)
    private val tunnelState = MutableStateFlow(TunnelState.Disconnected(null))

    private lateinit var viewModel: AddTimeViewModel

    @BeforeEach
    fun setUp() {
        every { mockPaymentUseCase.paymentAvailability } returns paymentAvailability
        every { mockPaymentUseCase.purchaseResult } returns purchaseResult
        every { mockConnectionProxy.tunnelState } returns tunnelState

        coEvery { mockPaymentUseCase.verifyPurchases() } returns
            VerificationResult.NothingToVerify.right()
        coEvery { mockPaymentUseCase.queryPaymentAvailability() } just Runs
        coEvery { mockPaymentUseCase.resetPurchaseResult() } just Runs
        coEvery { mockAccountRepository.getAccountData() } returns null

        viewModel =
            AddTimeViewModel(
                paymentUseCase = mockPaymentUseCase,
                accountRepository = mockAccountRepository,
                connectionProxy = mockConnectionProxy,
                isPlayBuild = true,
            )
    }

    @Test
    fun `when paymentAvailability emits ProductsUnavailable uiState should be NoPayment`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.emit(PaymentAvailability.ProductsUnavailable)
                val result = awaitItem()
                assertIs<Lc.Content<AddTimeUiState>>(result)
                assertIs<PaymentState.NoPayment>(result.value.billingPaymentState)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorOther uiState should be null`() = runTest {
        // Arrange
        paymentAvailability.emit(PaymentAvailability.Error.Other(mockk()))

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            val result = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(result)
            assertIs<PaymentState.Error.Generic>(result.value.billingPaymentState)
        }
    }

    @Test
    fun `when paymentAvailability emits ErrorBillingUnavailable uiState should be ErrorBilling`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.emit(PaymentAvailability.Error.BillingUnavailable)
                val result = awaitItem()
                assertIs<Lc.Content<AddTimeUiState>>(result)
                assertIs<PaymentState.Error.Billing>(result.value.billingPaymentState)
            }
        }

    @Test
    fun `when paymentAvailability emits ProductsAvailable uiState should be Available with products`() =
        runTest {
            // Arrange
            val mockProduct: PaymentProduct = mockk()
            val expectedProductList = listOf(mockProduct)

            // Act, Assert
            viewModel.uiState.test {
                awaitItem() // Default state
                paymentAvailability.emit(PaymentAvailability.ProductsAvailable(listOf(mockProduct)))
                val result = awaitItem()
                assertIs<Lc.Content<AddTimeUiState>>(result)
                assertIs<PaymentState.PaymentAvailable>(result.value.billingPaymentState)
                assertLists(expectedProductList, result.value.billingPaymentState.products)
            }
        }

    @Test
    fun `startBillingPayment should invoke purchaseProduct on PaymentUseCase`() {
        // Arrange
        val mockProductId = ProductId("MOCK")
        val mockActivityProvider = mockk<() -> Activity>()
        coEvery { mockPaymentUseCase.purchaseProduct(mockProductId, mockActivityProvider) } just
            Runs

        // Act
        viewModel.startBillingPayment(mockProductId, mockActivityProvider)

        // Assert
        coVerify { mockPaymentUseCase.purchaseProduct(mockProductId, mockActivityProvider) }
    }

    @Test
    fun `purchaseState success should invoke getAccountData on AccountRepository`() = runTest {
        // Arrange
        val purchaseResultData = PurchaseResult.Completed.Success(ProductId("one_month"))

        // Act
        purchaseResult.emit(purchaseResultData)

        // Assert
        coVerify { mockAccountRepository.getAccountData() }
    }

    @Test
    fun `purchaseState error should invoke queryPaymentAvailability on PaymentUseCase`() {
        // Arrange
        val purchaseResultData = PurchaseResult.Error.VerificationError(Throwable())

        // Act
        purchaseResult.value = purchaseResultData

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
    }

    @Test
    fun `resetPurchaseResult with success should invoke resetPurchaseResult on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.resetPurchaseResult()

        // Assert
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    @Test
    fun `purchaseResult emitting Success should result in success dialog state`() = runTest {
        // Arrange
        val productId = ProductId("one_month")
        val paymentProduct =
            PaymentProduct(productId = productId, price = ProductPrice("€5.00"), status = null)
        val result = PurchaseState.Success(productId)
        val purchaseResultData = PurchaseResult.Completed.Success(ProductId("one_month"))
        paymentAvailability.emit(PaymentAvailability.ProductsAvailable(listOf(paymentProduct)))

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            purchaseResult.emit(purchaseResultData)
            val item = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(item)
            assertEquals(result, item.value.purchaseState)
        }
    }
}
