package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.payment.model.PaymentAvailability
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.ReadableInstant
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class OutOfTimeViewModelTest {

    private val serviceConnectionStateFlow =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val accountExpiryStateFlow = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    private val deviceStateFlow = MutableStateFlow<DeviceState>(DeviceState.Initial)
    private val paymentAvailabilityFlow = MutableStateFlow<PaymentAvailability?>(null)
    private val purchaseResultFlow = MutableStateFlow<PurchaseResult?>(null)
    private val outOfTimeFlow = MutableStateFlow(true)

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val eventNotifierTunnelRealState =
        EventNotifier<TunnelState>(TunnelState.Disconnected())

    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)
    private val mockOutOfTimeUseCase: OutOfTimeUseCase = mockk(relaxed = true)

    private lateinit var viewModel: OutOfTimeViewModel

    @BeforeEach
    fun setUp() {
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)
        mockkStatic(PURCHASE_RESULT_EXTENSIONS_CLASS)

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionStateFlow

        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockConnectionProxy.onStateChange } returns eventNotifierTunnelRealState

        every { mockAccountRepository.accountExpiryState } returns accountExpiryStateFlow

        every { mockDeviceRepository.deviceState } returns deviceStateFlow

        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResultFlow

        coEvery { mockPaymentUseCase.paymentAvailability } returns paymentAvailabilityFlow

        coEvery { mockOutOfTimeUseCase.isOutOfTime() } returns outOfTimeFlow

        viewModel =
            OutOfTimeViewModel(
                accountRepository = mockAccountRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                deviceRepository = mockDeviceRepository,
                paymentUseCase = mockPaymentUseCase,
                outOfTimeUseCase = mockOutOfTimeUseCase,
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
    fun testSitePaymentClick() = runTest {
        // Arrange
        val mockToken = "4444 5555 6666 7777"
        val mockAuthTokenCache: AuthTokenCache = mockk(relaxed = true)
        every { mockServiceConnectionManager.authTokenCache() } returns mockAuthTokenCache
        coEvery { mockAuthTokenCache.fetchAuthToken() } returns mockToken

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onSitePaymentClick()
            val action = awaitItem()
            assertIs<OutOfTimeViewModel.UiSideEffect.OpenAccountView>(action)
            assertEquals(mockToken, action.token)
        }
    }

    @Test
    fun testUpdateTunnelState() = runTest {
        // Arrange
        val tunnelRealStateTestItem = TunnelState.Connected(mockk(), mockk())

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(OutOfTimeUiState(deviceName = ""), awaitItem())
            eventNotifierTunnelRealState.notify(tunnelRealStateTestItem)
            serviceConnectionStateFlow.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val result = awaitItem()
            assertEquals(tunnelRealStateTestItem, result.tunnelState)
        }
    }

    @Test
    fun testOpenConnectScreen() = runTest {
        // Arrange
        val mockExpiryDate: DateTime = mockk()
        every { mockExpiryDate.isAfter(any<ReadableInstant>()) } returns true

        // Act, Assert
        viewModel.uiSideEffect.test {
            outOfTimeFlow.value = false
            val action = awaitItem()
            assertIs<OutOfTimeViewModel.UiSideEffect.OpenConnectScreen>(action)
        }
    }

    @Test
    fun testOnDisconnectClick() = runTest {
        // Arrange
        val mockProxy: ConnectionProxy = mockk(relaxed = true)
        every { mockServiceConnectionManager.connectionProxy() } returns mockProxy

        // Act
        viewModel.onDisconnectClick()

        // Assert
        verify { mockProxy.disconnect() }
    }

    @Test
    fun testBillingProductsUnavailableState() = runTest {
        // Arrange
        val productsUnavailable = PaymentAvailability.ProductsUnavailable
        paymentAvailabilityFlow.value = productsUnavailable
        serviceConnectionStateFlow.value =
            ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

        // Act, Assert
        viewModel.uiState.test {
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.NoPayment>(result)
        }
    }

    @Test
    fun testBillingProductsGenericErrorState() = runTest {
        // Arrange
        val paymentAvailabilityError = PaymentAvailability.Error.Other(mockk())
        paymentAvailabilityFlow.value = paymentAvailabilityError
        serviceConnectionStateFlow.value =
            ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

        // Act, Assert
        viewModel.uiState.test {
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.Error.Generic>(result)
        }
    }

    @Test
    fun testBillingProductsBillingErrorState() = runTest {
        // Arrange
        val paymentAvailabilityError = PaymentAvailability.Error.BillingUnavailable
        paymentAvailabilityFlow.value = paymentAvailabilityError
        serviceConnectionStateFlow.value =
            ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)

        // Act, Assert
        viewModel.uiState.test {
            val result = awaitItem().billingPaymentState
            assertIs<PaymentState.Error.Billing>(result)
        }
    }

    @Test
    fun testBillingProductsPaymentAvailableState() = runTest {
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

    @Test
    fun testOnClosePurchaseResultDialogSuccessful() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = true)

        // Assert
        verify { mockAccountRepository.fetchAccountExpiry() }
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    @Test
    fun testOnClosePurchaseResultDialogNotSuccessful() {
        // Arrange

        // Act
        viewModel.onClosePurchaseResultDialog(success = false)

        // Assert
        coVerify { mockPaymentUseCase.queryPaymentAvailability() }
        coVerify { mockPaymentUseCase.resetPurchaseResult() }
    }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
        private const val PURCHASE_RESULT_EXTENSIONS_CLASS =
            "net.mullvad.mullvadvpn.util.PurchaseResultExtensionsKt"
    }
}
