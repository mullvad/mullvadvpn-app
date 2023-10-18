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
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountAndDevice
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.Device
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
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.ReadableInstant
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class WelcomeViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val deviceState = MutableStateFlow<DeviceState>(DeviceState.Initial)
    private val accountExpiryState = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected)

    private val mockAccountRepository: AccountRepository = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()

    private lateinit var viewModel: WelcomeViewModel

    @Before
    fun setUp() {
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)

        every { mockDeviceRepository.deviceState } returns deviceState

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState

        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState

        every { mockAccountRepository.accountExpiryState } returns accountExpiryState

        viewModel =
            WelcomeViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                serviceConnectionManager = mockServiceConnectionManager,
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
                assertIs<WelcomeViewModel.UiSideEffect.OpenAccountView>(action)
                assertEquals(mockToken, action.token)
            }
        }

    @Test
    fun testUpdateTunnelState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val tunnelUiStateTestItem = TunnelState.Connected(mockk(), mockk())

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(WelcomeUiState(), awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                eventNotifierTunnelUiState.notify(tunnelUiStateTestItem)
                val result = awaitItem()
                assertEquals(tunnelUiStateTestItem, result.tunnelState)
            }
        }

    @Test
    fun testUpdateAccountNumber() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val expectedAccountNumber = "4444555566667777"
            val device: Device = mockk()
            every { device.displayName() } returns ""

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(WelcomeUiState(), awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                deviceState.value =
                    DeviceState.LoggedIn(
                        accountAndDevice =
                            AccountAndDevice(account_token = expectedAccountNumber, device = device)
                    )
                val result = awaitItem()
                assertEquals(expectedAccountNumber, result.accountNumber)
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
                assertIs<WelcomeViewModel.UiSideEffect.OpenConnectScreen>(action)
            }
        }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
