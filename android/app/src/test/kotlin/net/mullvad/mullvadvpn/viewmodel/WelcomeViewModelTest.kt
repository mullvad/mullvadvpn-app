package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
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
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class WelcomeViewModelTest {

    private val serviceConnectionStateFlow =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Unbound)
    private val deviceStateFlow = MutableStateFlow<DeviceState>(DeviceState.Initial)
    private val accountExpiryStateFlow = MutableStateFlow<AccountData>(AccountData.Missing)
    private val purchaseResultFlow = MutableStateFlow<PurchaseResult?>(null)
    private val paymentAvailabilityFlow = MutableStateFlow<PaymentAvailability?>(null)

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected())

    private val mockAccountRepository: net.mullvad.mullvadvpn.lib.account.AccountRepository =
        mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    private lateinit var viewModel: WelcomeViewModel

    @BeforeEach
    fun setup() {
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)
        mockkStatic(PURCHASE_RESULT_EXTENSIONS_CLASS)

        every { mockDeviceRepository.deviceState } returns deviceStateFlow

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionStateFlow

        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState

        every { mockAccountRepository.accountExpiryState } returns accountExpiryStateFlow

        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResultFlow

        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailabilityFlow

        viewModel =
            WelcomeViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                paymentUseCase = mockPaymentUseCase,
                pollAccountExpiry = false,
                isPlayBuild = false
            )
    }

    @AfterEach
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `on onSitePaymentClick call uiSideEffect should emit OpenAccountView`() = runTest {
        // Arrange
        val mockToken = "4444 5555 6666 7777"
        val mockAuthTokenCache: AuthTokenCache = mockk(relaxed = true)
        every { mockServiceConnectionManager.authTokenCache() } returns mockAuthTokenCache
        coEvery { mockAuthTokenCache.fetchAuthToken() } returns mockToken

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onSitePaymentClick()
            val action = awaitItem()
            assertIs<WelcomeViewModel.UiSideEffect.OpenAccountView>(action)
            assertEquals(mockToken, action.token)
        }
    }

    @Test
    fun `on new TunnelState uiState should include new TunnelState`() = runTest {
        // Arrange
        val tunnelUiStateTestItem: TunnelState = mockk()

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(WelcomeUiState(), awaitItem())
            eventNotifierTunnelUiState.notify(tunnelUiStateTestItem)
            serviceConnectionStateFlow.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val result = awaitItem()
            assertEquals(tunnelUiStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun `when DeviceRepository returns LoggedIn uiState should include new accountNumber`() =
        runTest {
            // Arrange
            val expectedAccountNumber = "4444555566667777"
            val device: Device = mockk()
            every { device.displayName() } returns ""

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(WelcomeUiState(), awaitItem())
                paymentAvailabilityFlow.value = null
                deviceStateFlow.value =
                    DeviceState.LoggedIn(
                        accountAndDevice =
                            AccountAndDevice(account_token = expectedAccountNumber, device = device)
                    )
                serviceConnectionStateFlow.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                assertEquals(expectedAccountNumber, awaitItem().accountNumber)
            }
        }

    @Test
    fun `when user has added time then uiSideEffect should emit OpenConnectScreen`() = runTest {
        // Arrange
        accountExpiryStateFlow.emit(AccountData.Available(DateTime().plusDays(1)))

        // Act, Assert
        viewModel.uiSideEffect.test {
            val action = awaitItem()
            assertIs<WelcomeViewModel.UiSideEffect.OpenConnectScreen>(action)
        }
    }

    @Test
    fun `when paymentAvailability emits ProductsUnavailable uiState should include state NoPayment`() =
        runTest {
            // Arrange
            val productsUnavailable = PaymentAvailability.ProductsUnavailable

            // Act, Assert
            viewModel.uiState.test {
                // Default item
                awaitItem()
                paymentAvailabilityFlow.tryEmit(productsUnavailable)
                serviceConnectionStateFlow.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.NoPayment>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorOther uiState should include state ErrorGeneric`() =
        runTest {
            // Arrange
            val paymentOtherError = PaymentAvailability.Error.Other(mockk())
            paymentAvailabilityFlow.tryEmit(paymentOtherError)
            serviceConnectionStateFlow.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Generic>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ErrorBillingUnavailable uiState should include state ErrorBilling`() =
        runTest { // Arrange
            val paymentBillingError = PaymentAvailability.Error.BillingUnavailable
            paymentAvailabilityFlow.value = paymentBillingError
            serviceConnectionStateFlow.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.Error.Billing>(result)
            }
        }

    @Test
    fun `when paymentAvailability emits ProductsAvailable uiState should include state Available with products`() =
        runTest {
            // Arrange
            val mockProduct: PaymentProduct = mockk()
            val expectedProductList = listOf(mockProduct)
            val productsAvailable = PaymentAvailability.ProductsAvailable(listOf(mockProduct))
            paymentAvailabilityFlow.value = productsAvailable
            serviceConnectionStateFlow.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

            // Act, Assert
            viewModel.uiState.test {
                val result = awaitItem().billingPaymentState
                assertIs<PaymentState.PaymentAvailable>(result)
                assertLists(expectedProductList, result.products)
            }
        }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
    }
}
