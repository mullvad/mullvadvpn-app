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
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.payment.model.VerificationResult
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.Lc
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
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
        coEvery { mockAccountRepository.refreshAccountData(any()) } just Runs

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

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state

            // Emit an error with a mock exception
            paymentAvailability.emit(PaymentAvailability.Error.Other(mockk()))

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
    fun `startBillingPayment should invoke purchaseProduct on PaymentUseCase`() = runTest {
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
        coVerify { mockAccountRepository.refreshAccountData(ignoreTimeout = true) }
    }

    @Test
    fun `purchaseState error should invoke queryPaymentAvailability on PaymentUseCase`() = runTest {
        // Arrange
        val purchaseResultData = PurchaseResult.Error.VerificationError(Throwable())

        // Act
        purchaseResult.value = purchaseResultData

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
    }

    @Test
    fun `resetPurchaseResult with success should invoke resetPurchaseResult on PaymentUseCase`() =
        runTest {
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

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            paymentAvailability.emit(PaymentAvailability.ProductsAvailable(listOf(paymentProduct)))
            awaitItem()
            purchaseResult.emit(purchaseResultData)
            val item = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(item)
            assertEquals(result, item.value.purchaseState)
        }
    }

    @Test
    fun `purchaseResult error billing error should result in purchase state null`() = runTest {
        // Arrange
        val productId = ProductId("one_month")
        val paymentProduct =
            PaymentProduct(productId = productId, price = ProductPrice("€5.00"), status = null)
        val purchaseResultLoading = PurchaseResult.FetchingProducts
        val purchaseResultData = PurchaseResult.Error.BillingError(null)

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            // Payment availability can not be null, otherwise the test will timeout
            paymentAvailability.emit(PaymentAvailability.ProductsAvailable(listOf(paymentProduct)))
            awaitItem()
            purchaseResult.emit(
                purchaseResultLoading
            ) // Set up loading state so we get a new state when we emit the error
            val loadingItem = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(loadingItem)
            assertEquals(PurchaseState.Connecting, loadingItem.value.purchaseState)
            purchaseResult.emit(purchaseResultData)
            val item = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(item)
            assertEquals(null, item.value.purchaseState)
        }
    }

    @Test
    fun `purchaseResult cancelled should result in purchase state null`() = runTest {
        // Arrange
        val productId = ProductId("one_month")
        val paymentProduct =
            PaymentProduct(productId = productId, price = ProductPrice("€5.00"), status = null)
        val purchaseResultLoading = PurchaseResult.FetchingProducts
        val purchaseResultData = PurchaseResult.Completed.Cancelled

        // Act, Assert
        viewModel.uiState.test {
            awaitItem() // Default state
            // Payment availability can not be null, otherwise the test will timeout
            paymentAvailability.emit(PaymentAvailability.ProductsAvailable(listOf(paymentProduct)))
            awaitItem()
            purchaseResult.emit(
                purchaseResultLoading
            ) // Set up loading state so we get a new state when we emit cancelled
            val loadingItem = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(loadingItem)
            assertEquals(PurchaseState.Connecting, loadingItem.value.purchaseState)
            purchaseResult.emit(purchaseResultData)
            val item = awaitItem()
            assertIs<Lc.Content<AddTimeUiState>>(item)
            assertEquals(null, item.value.purchaseState)
        }
    }
}
