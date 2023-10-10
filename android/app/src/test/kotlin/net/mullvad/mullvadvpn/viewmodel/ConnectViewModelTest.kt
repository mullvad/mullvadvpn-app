package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.ConnectNotificationState
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.AuthTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime
import org.joda.time.ReadableInstant
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ConnectViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

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

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockLocationInfoCache: LocationInfoCache = mockk(relaxUnitFun = true)
    private val mockRelayListListener: RelayListListener = mockk(relaxUnitFun = true)
    private lateinit var mockAppVersionInfoCache: AppVersionInfoCache
    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val mockLocation: GeoIpLocation = mockk(relaxed = true)

    // Account Repository
    private val mockAccountRepository: AccountRepository = mockk()

    // Device Repository
    private val mockDeviceRepository: DeviceRepository = mockk()

    // Captures
    private val locationSlot = slot<((GeoIpLocation?) -> Unit)>()
    private val relaySlot = slot<(List<RelayCountry>, RelayItem?) -> Unit>()

    // Event notifiers
    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected)
    private val eventNotifierTunnelRealState = EventNotifier<TunnelState>(TunnelState.Disconnected)

    @Before
    fun setup() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)

        mockAppVersionInfoCache =
            mockk<AppVersionInfoCache>().apply {
                every { appVersionCallbackFlow() } returns versionInfo
            }

        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.locationInfoCache } returns mockLocationInfoCache
        every { mockServiceConnectionContainer.relayListListener } returns mockRelayListListener
        every { mockServiceConnectionContainer.appVersionInfoCache } returns mockAppVersionInfoCache
        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        every { mockAccountRepository.accountExpiryState } returns accountExpiryState

        every { mockDeviceRepository.deviceState } returns deviceState

        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState
        every { mockConnectionProxy.onStateChange } returns eventNotifierTunnelRealState

        every { mockLocation.country } returns "dummy country"

        // Listeners
        every { mockLocationInfoCache.onNewLocation = capture(locationSlot) } answers {}
        every { mockRelayListListener.onRelayCountriesChange = capture(relaySlot) } answers {}
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}

        viewModel =
            ConnectViewModel(
                serviceConnectionManager = mockServiceConnectionManager,
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                isVersionInfoNotificationEnabled = true
            )
    }

    @After
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testInitialState() = runTest {
        viewModel.uiState.test { assertEquals(ConnectUiState.INITIAL, awaitItem()) }
    }

    @Test
    fun testTunnelInfoExpandedUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedResult = true

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                viewModel.toggleTunnelInfoExpansion()
                val result = awaitItem()
                assertEquals(expectedResult, result.isTunnelInfoExpanded)
            }
        }

    @Test
    fun testTunnelRealStateUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val tunnelRealStateTestItem = TunnelState.Connected(mockk(relaxed = true), mockk())

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelRealState.notify(tunnelRealStateTestItem)
                val result = awaitItem()
                assertEquals(tunnelRealStateTestItem, result.tunnelRealState)
            }
        }

    @Test
    fun testTunnelUiStateUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val tunnelUiStateTestItem = TunnelState.Connected(mockk(), mockk())

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelUiState.notify(tunnelUiStateTestItem)
                val result = awaitItem()
                assertEquals(tunnelUiStateTestItem, result.tunnelUiState)
            }
        }

    @Test
    fun testRelayItemUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val relayTestItem =
                RelayCountry(name = "Name", code = "Code", expanded = false, cities = emptyList())

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), relayTestItem)
                val result = awaitItem()
                assertEquals(relayTestItem, result.relayLocation)
            }
        }

    @Test
    fun testLocationUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val locationTestItem =
                GeoIpLocation(
                    ipv4 = mockk(relaxed = true),
                    ipv6 = mockk(relaxed = true),
                    country = "Sweden",
                    city = "Gothenburg",
                    hostname = "Host"
                )

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(locationTestItem)
                relaySlot.captured.invoke(mockk(), mockk())
                val result = awaitItem()
                assertEquals(locationTestItem, result.location)
            }
        }

    @Test
    fun testLocationUpdateNullLocation() =
        // Arrange
        runTest(testCoroutineRule.testDispatcher) {
            val locationTestItem = null

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(locationTestItem)
                relaySlot.captured.invoke(mockk(), mockk())
                expectNoEvents()
                val result = awaitItem()
                assertEquals(locationTestItem, result.location)
            }
        }

    @Test
    fun testOnDisconnectClick() =
        runTest(testCoroutineRule.testDispatcher) {
            val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
            every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
            viewModel.onDisconnectClick()
            verify { mockConnectionProxy.disconnect() }
        }

    @Test
    fun testOnReconnectClick() =
        runTest(testCoroutineRule.testDispatcher) {
            val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
            every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
            viewModel.onReconnectClick()
            verify { mockConnectionProxy.reconnect() }
        }

    @Test
    fun testOnConnectClick() =
        runTest(testCoroutineRule.testDispatcher) {
            val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
            every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
            viewModel.onConnectClick()
            verify { mockConnectionProxy.connect() }
        }

    @Test
    fun testOnCancelClick() =
        runTest(testCoroutineRule.testDispatcher) {
            val mockConnectionProxy: ConnectionProxy = mockk(relaxed = true)
            every { mockServiceConnectionManager.connectionProxy() } returns mockConnectionProxy
            viewModel.onCancelClick()
            verify { mockConnectionProxy.disconnect() }
        }

    @Test
    fun testBlockingNotificationState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val expectedConnectNotificationState =
                ConnectNotificationState.ShowTunnelStateNotificationBlocked
            val tunnelUiState = TunnelState.Connecting(null, null)

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelUiState.notify(tunnelUiState)
                val result = awaitItem()
                assertEquals(expectedConnectNotificationState, result.connectNotificationState)
            }
        }

    @Test
    fun testErrorNotificationState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val mockErrorState: ErrorState = mockk()
            val expectedConnectNotificationState =
                ConnectNotificationState.ShowTunnelStateNotificationError(mockErrorState)
            val tunnelUiState = TunnelState.Error(mockErrorState)

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelUiState.notify(tunnelUiState)
                val result = awaitItem()
                assertEquals(expectedConnectNotificationState, result.connectNotificationState)
            }
        }

    @Test
    fun testVersionInfoNotificationState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val mockVersionInfo: VersionInfo = mockk()
            val expectedConnectNotificationState =
                ConnectNotificationState.ShowVersionInfoNotification(mockVersionInfo)
            every { mockVersionInfo.isOutdated } returns true

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                versionInfo.value = mockVersionInfo
                val result = awaitItem()
                assertEquals(expectedConnectNotificationState, result.connectNotificationState)
            }
        }

    @Test
    fun testAccountExpiryNotificationState() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val mockDateTime: DateTime = mockk()
            val expectedConnectNotificationState =
                ConnectNotificationState.ShowAccountExpiryNotification(mockDateTime)
            every { mockDateTime.isBefore(any<ReadableInstant>()) } returns true
            every { mockDateTime.toInstant().millis } returns 0

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                accountExpiryState.value = AccountExpiry.Available(mockDateTime)

                val result = awaitItem()
                assertEquals(expectedConnectNotificationState, result.connectNotificationState)
            }
        }

    @Test
    fun testOnShowAccountClick() =
        runTest(testCoroutineRule.testDispatcher) {
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
    fun testOutOfTimeUiSideEffect() =
        runTest(testCoroutineRule.testDispatcher) {
            // Arrange
            val errorStateCause = ErrorStateCause.AuthFailed("[EXPIRED_ACCOUNT]")
            val tunnelRealStateTestItem = TunnelState.Error(ErrorState(errorStateCause, true))
            val deferred = async { viewModel.uiSideEffect.first() }

            // Act
            viewModel.uiState.test {
                awaitItem()
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockLocation)
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelRealState.notify(tunnelRealStateTestItem)
                awaitItem()
            }

            // Assert
            assertIs<ConnectViewModel.UiSideEffect.OpenOutOfTimeView>(deferred.await())
        }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
