package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
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
import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
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
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.ReadableInstant
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class OutOfTimeViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val accountExpiryState = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    private val deviceState = MutableStateFlow<DeviceState>(DeviceState.Initial)

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val eventNotifierTunnelRealState = EventNotifier<TunnelState>(TunnelState.Disconnected)

    private val mockAccountRepository: AccountRepository = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockPaymentProvider: PaymentProvider = mockk(relaxed = true)

    private lateinit var viewModel: OutOfTimeViewModel

    @Before
    fun setUp() {
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState

        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockConnectionProxy.onStateChange } returns eventNotifierTunnelRealState

        every { mockAccountRepository.accountExpiryState } returns accountExpiryState

        every { mockDeviceRepository.deviceState } returns deviceState

        viewModel =
            OutOfTimeViewModel(
                accountRepository = mockAccountRepository,
                serviceConnectionManager = mockServiceConnectionManager,
                deviceRepository = mockDeviceRepository,
                paymentProvider = mockPaymentProvider,
                pollAccountExpiry = false
            )
    }

    @After
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testSitePaymentClick() =
        runTest(testCoroutineRule.testDispatcher) {
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
    fun testUpdateTunnelState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val tunnelRealStateTestItem = TunnelState.Connected(mockk(), mockk())

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(OutOfTimeUiState(deviceName = ""), awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                eventNotifierTunnelRealState.notify(tunnelRealStateTestItem)
                val result = awaitItem()
                assertEquals(tunnelRealStateTestItem, result.tunnelState)
            }
        }

    @Test
    fun testOpenConnectScreen() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val mockExpiryDate: DateTime = mockk()
            every { mockExpiryDate.isAfter(any<ReadableInstant>()) } returns true

            // Act, Assert
            viewModel.uiSideEffect.test {
                accountExpiryState.value = AccountExpiry.Available(mockExpiryDate)
                val action = awaitItem()
                assertIs<OutOfTimeViewModel.UiSideEffect.OpenConnectScreen>(action)
            }
        }

    @Test
    fun testOnDisconnectClick() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val mockProxy: ConnectionProxy = mockk(relaxed = true)
            every { mockServiceConnectionManager.connectionProxy() } returns mockProxy

            // Act
            viewModel.onDisconnectClick()

            // Assert
            verify { mockProxy.disconnect() }
        }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
