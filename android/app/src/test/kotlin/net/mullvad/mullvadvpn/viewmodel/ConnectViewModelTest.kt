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
import kotlin.test.assertNull
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.SelectedLocation
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.util.EventNotifier
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class ConnectViewModelTest {

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var viewModel: ConnectViewModel

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val versionInfo =
        MutableStateFlow(
            VersionInfo(
                currentVersion = null,
                upgradeVersion = null,
                isOutdated = false,
                isSupported = true
            )
        )
    private val accountExpiryState = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    private val deviceState = MutableStateFlow<DeviceState>(DeviceState.Initial)
    private val notifications = MutableStateFlow<List<InAppNotification>>(emptyList())

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private lateinit var mockAppVersionInfoCache: AppVersionInfoCache
    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockLocation: GeoIpLocation = mockk(relaxed = true)

    // Account Repository
    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)

    // Device Repository
    private val mockDeviceRepository: DeviceRepository = mockk()

    // In App Notifications
    private val mockInAppNotificationController: InAppNotificationController = mockk()

    // Relay list use case
    private val mockRelayListUseCase: RelayListUseCase = mockk()

    // Payment use case
    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    // Event notifiers
    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected())
    private val eventNotifierTunnelRealState =
        EventNotifier<TunnelState>(TunnelState.Disconnected())

    // Flows
    private val selectedLocationFlow = MutableStateFlow<SelectedLocation?>(null)

    // Out Of Time Use Case
    private val outOfTimeUseCase: OutOfTimeUseCase = mockk()
    private val outOfTimeViewFlow = MutableStateFlow(false)

    @BeforeEach
    fun setup() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)

        mockAppVersionInfoCache =
            mockk<AppVersionInfoCache>().apply {
                every { appVersionCallbackFlow() } returns versionInfo
            }

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockAccountRepository.accountExpiryState } returns accountExpiryState

        every { mockDeviceRepository.deviceState } returns deviceState

        every { mockInAppNotificationController.notifications } returns notifications

        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState
        every { mockConnectionProxy.onStateChange } returns eventNotifierTunnelRealState

        every { mockLocation.country } returns "dummy country"

        // Listeners
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}

        // Flows
        every { mockRelayListUseCase.selectedLocation() } returns selectedLocationFlow

        every { outOfTimeUseCase.isOutOfTime() } returns outOfTimeViewFlow
        viewModel =
            ConnectViewModel(
                serviceConnectionManager = mockServiceConnectionManager,
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                inAppNotificationController = mockInAppNotificationController,
                relayListUseCase = mockRelayListUseCase,
                newDeviceNotificationUseCase = mockk(),
                outOfTimeUseCase = outOfTimeUseCase,
                paymentUseCase = mockPaymentUseCase,
                isPlayBuild = false
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testInitialState() = runTest {
        viewModel.uiState.test { assertEquals(ConnectUiState.INITIAL, awaitItem()) }
    }

    @Test
    fun testTunnelRealStateUpdate() = runTest {
        val tunnelRealStateTestItem = TunnelState.Connected(mockk(relaxed = true), null)

        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            eventNotifierTunnelRealState.notify(tunnelRealStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelRealStateTestItem, result.tunnelRealState)
        }
    }

    @Test
    fun testTunnelUiStateUpdate() = runTest {
        val tunnelUiStateTestItem = TunnelState.Connected(mockk(), mockk())

        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            eventNotifierTunnelUiState.notify(tunnelUiStateTestItem)
            val result = awaitItem()
            assertEquals(tunnelUiStateTestItem, result.tunnelUiState)
        }
    }

    @Test
    fun testSelectedLocationUpdate() = runTest {
        val selectedLocation =
            SelectedLocation(
                id = "MOCK-ID",
                name = "MOCK-NAME",
                geographicLocationConstraint =
                    GeographicLocationConstraint.Country(countryCode = "MOCK-COUNTRY-CODE")
            )
        selectedLocationFlow.value = selectedLocation

        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            val result = awaitItem()
            assertEquals(selectedLocation, result.selectedLocation)
        }
    }

    @Test
    fun testLocationUpdate() = runTest {
        val locationTestItem =
            GeoIpLocation(
                ipv4 = mockk(relaxed = true),
                ipv6 = mockk(relaxed = true),
                country = "Sweden",
                city = "Gothenburg",
                hostname = "Host"
            )

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            eventNotifierTunnelRealState.notify(TunnelState.Disconnected(null))

            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            // Start of with no location
            assertNull(awaitItem().location)

            // After updated we show latest
            eventNotifierTunnelRealState.notify(TunnelState.Disconnected(locationTestItem))
            assertEquals(locationTestItem, awaitItem().location)
        }
    }

    @Test
    fun testLocationUpdateNullLocation() =
        // Arrange
        runTest {
            val locationTestItem = null

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                expectNoEvents()
                val result = awaitItem()
                assertEquals(locationTestItem, result.location)
            }
        }

    @Test
    fun testOnDisconnectClick() = runTest {
        val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
        every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
        viewModel.onDisconnectClick()
        verify { mockConnectionProxy.disconnect() }
    }

    @Test
    fun testOnReconnectClick() = runTest {
        val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
        every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
        viewModel.onReconnectClick()
        verify { mockConnectionProxy.reconnect() }
    }

    @Test
    fun testOnConnectClick() = runTest {
        val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
        every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
        viewModel.onConnectClick()
        verify { mockConnectionProxy.connect() }
    }

    @Test
    fun testOnCancelClick() = runTest {
        val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
        every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
        viewModel.onCancelClick()
        verify { mockConnectionProxy.disconnect() }
    }

    @Test
    fun testErrorNotificationState() = runTest {
        // Arrange
        val mockErrorState: ErrorState = mockk()
        val expectedConnectNotificationState = InAppNotification.TunnelStateError(mockErrorState)
        val tunnelUiState = TunnelState.Error(mockErrorState)
        notifications.value = listOf(expectedConnectNotificationState)

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ConnectUiState.INITIAL, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            eventNotifierTunnelUiState.notify(tunnelUiState)
            val result = awaitItem()
            assertEquals(expectedConnectNotificationState, result.inAppNotification)
        }
    }

    @Test
    fun testOnShowAccountClick() = runTest {
        // Arrange
        val mockToken = "4444 5555 6666 7777"
        val mockAuthTokenCache: AuthTokenCache = mockk(relaxed = true)
        every { mockServiceConnectionManager.authTokenCache() } returns mockAuthTokenCache
        coEvery { mockAuthTokenCache.fetchAuthToken() } returns mockToken

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.onManageAccountClick()
            val action = awaitItem()
            assertIs<ConnectViewModel.UiSideEffect.OpenAccountManagementPageInBrowser>(action)
            assertEquals(mockToken, action.token)
        }
    }

    @Test
    fun testOutOfTimeUiSideEffect() = runTest {
        // Arrange
        val deferred = async { viewModel.uiSideEffect.first() }

        // Act
        viewModel.uiState.test {
            awaitItem()
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            outOfTimeViewFlow.value = true
            awaitItem()
        }

        // Assert
        assertIs<ConnectViewModel.UiSideEffect.OutOfTime>(deferred.await())
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
