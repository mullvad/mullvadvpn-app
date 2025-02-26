package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class OutOfTimeViewModelTest {

    private val serviceConnectionStateFlow =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)
    private val accountExpiryStateFlow = MutableStateFlow<AccountData?>(null)
    private val accountStateFlow = MutableStateFlow<DeviceState?>(null)
    private val paymentAvailabilityFlow = MutableStateFlow<PaymentAvailability?>(null)
    private val purchaseResultFlow = MutableStateFlow<PurchaseResult?>(null)
    private val outOfTimeFlow = MutableStateFlow(true)

    // Connection Proxy
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())

    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk(relaxed = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)
    private val mockOutOfTimeUseCase: OutOfTimeUseCase = mockk(relaxed = true)

    private lateinit var viewModel: OutOfTimeViewModel

    @BeforeEach
    fun setup() {
        mockkStatic(PURCHASE_RESULT_EXTENSIONS_CLASS)

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionStateFlow

        every { mockConnectionProxy.tunnelState } returns tunnelState

        every { mockAccountRepository.accountData } returns accountExpiryStateFlow

        every { mockDeviceRepository.deviceState } returns accountStateFlow

        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResultFlow

        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailabilityFlow

        coEvery { mockOutOfTimeUseCase.isOutOfTime } returns outOfTimeFlow

        viewModel =
            OutOfTimeViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                paymentUseCase = mockPaymentUseCase,
                outOfTimeUseCase = mockOutOfTimeUseCase,
                connectionProxy = mockConnectionProxy,
                pollAccountExpiry = false,
                isPlayBuild = false,
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `when clicking on site payment then open website account view`() = runTest {
        // Arrange
        val mockToken = WebsiteAuthToken.fromString("154c4cc94810fddac78398662b7fa0c7")
        coEvery { mockAccountRepository.getWebsiteAuthToken() } returns mockToken

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onSitePaymentClick()
            val action = awaitItem()
            assertIs<OutOfTimeViewModel.UiSideEffect.OpenAccountView>(action)
            assertEquals(mockToken, action.token)
        }
    }

    @Test
    fun `when tunnel state changes then ui should be updated`() = runTest {
        // Arrange
        val tunnelRealStateTestItem = TunnelState.Connected(mockk(), mockk(), emptyList())

        // Act, Assert
        viewModel.uiState.test {
            // Default item
            awaitItem()
            tunnelState.emit(tunnelRealStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelRealStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun `when OutOfTimeUseCase returns false uiSideEffect should emit OpenConnectScreen`() =
        runTest {
            // Act, Assert
            viewModel.uiSideEffect.test {
                outOfTimeFlow.value = false
                val action = awaitItem()
                assertIs<OutOfTimeViewModel.UiSideEffect.OpenConnectScreen>(action)
            }
        }

    @Test
    fun `onDisconnectClick should invoke disconnect on ConnectionProxy`() = runTest {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()

        // Act
        viewModel.onDisconnectClick()

        // Assert
        coVerify { mockConnectionProxy.disconnect() }
    }

    @Test
    fun `when paymentAvailability emits ProductsUnavailable uiState should include state NoPayment`() =
        runTest {
            // Arrange
            val productsUnavailable = PaymentAvailability.ProductsUnavailable
            paymentAvailabilityFlow.value = productsUnavailable

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.NoPayment>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorOther uiState should include state ErrorGeneric`() =
        runTest {
            // Arrange
            val paymentAvailabilityError = PaymentAvailability.Error.Other(mockk())
            paymentAvailabilityFlow.value = paymentAvailabilityError

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Generic>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorBillingUnavailable uiState should be ErrorBilling`() =
        runTest {
            // Arrange
            val paymentAvailabilityError = PaymentAvailability.Error.BillingUnavailable
            paymentAvailabilityFlow.value = paymentAvailabilityError

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Billing>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ProductsAvailable uiState should be Available with products`() =
        runTest {
            // Arrange
            val mockProduct: PaymentProduct = mockk()
            val expectedProductList = listOf(mockProduct)
            val productsAvailable = PaymentAvailability.ProductsAvailable(listOf(mockProduct))
            paymentAvailabilityFlow.value = productsAvailable

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.PaymentAvailable>(result)
                assertLists(expectedProductList, result.products)
            }
        }

    @Test
    fun `onClosePurchaseResultDialog with success should invoke getAccountData on AccountRepository`() {
        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        coVerify { mockAccountRepository.getAccountData() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success should invoke resetPurchaseResult on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success false should invoke queryPaymentAvailability on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
    }

    @Test
    fun `onClosePurchaseResultDialog with success false should invoke resetPurchaseResult on PaymentUseCase`() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    companion object {
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
    }
}
