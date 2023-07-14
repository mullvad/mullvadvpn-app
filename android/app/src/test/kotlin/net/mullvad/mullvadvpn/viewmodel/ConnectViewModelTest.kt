package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import net.mullvad.talpid.util.EventNotifier
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
                isSupported = false
            )
        )

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockLocationInfoCache: LocationInfoCache = mockk(relaxUnitFun = true)
    private val mockRelayListListener: RelayListListener = mockk(relaxUnitFun = true)
    private lateinit var mockAppVersionInfoCache: AppVersionInfoCache
    private val mockConnectionProxy: ConnectionProxy = mockk()

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

        every { mockConnectionProxy.onUiStateChange } returns eventNotifierTunnelUiState
        every { mockConnectionProxy.onStateChange } returns eventNotifierTunnelRealState
        // Listeners
        every { mockLocationInfoCache.onNewLocation = capture(locationSlot) } answers {}
        every { mockRelayListListener.onRelayCountriesChange = capture(relaySlot) } answers {}
        every { mockAppVersionInfoCache.onUpdate = any() } answers {}

        viewModel = ConnectViewModel(mockServiceConnectionManager)
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
                locationSlot.captured.invoke(mockk(relaxed = true))
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
                locationSlot.captured.invoke(mockk(relaxed = true))
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
                locationSlot.captured.invoke(mockk(relaxed = true))
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelUiState.notify(tunnelUiStateTestItem)
                val result = awaitItem()
                assertEquals(tunnelUiStateTestItem, result.tunnelUiState)
            }
        }

    @Test
    fun testAppVersionInfoUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val versionInfoTestItem =
                VersionInfo(
                    currentVersion = "1.0",
                    upgradeVersion = "2.0",
                    isOutdated = false,
                    isSupported = false
                )

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockk(relaxed = true))
                relaySlot.captured.invoke(mockk(), mockk())
                versionInfo.value = versionInfoTestItem
                val result = awaitItem()
                assertEquals(versionInfoTestItem, result.versionInfo)
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
                locationSlot.captured.invoke(mockk(relaxed = true))
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

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
