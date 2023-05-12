package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import java.net.InetAddress
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.TestCoroutineRule
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.LocationInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
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
    private val relaySlot = slot<(RelayList, RelayItem?) -> Unit>()

    // Event notifiers
    private val eventNotifierTunnelUiState = EventNotifier<TunnelState>(TunnelState.Disconnected)
    private val eventNotifierTunnelRealState = EventNotifier<TunnelState>(TunnelState.Disconnected)

    @Before
    fun setup() {
        mockkStatic(CACHE_EXTENSION_CLASS)

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
        every { mockRelayListListener.onRelayListChange = capture(relaySlot) } answers {}
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
                locationSlot.captured.invoke(mockk())
                relaySlot.captured.invoke(mockk(), mockk())
                viewModel.toggleTunnelInfoExpansion()
                val result = awaitItem()
                assertEquals(expectedResult, result.isTunnelInfoExpanded)
            }
        }

    @Test
    fun testTunnelRealStateUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedResult = TunnelState.Connected(mockk(), mockk())

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockk())
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelRealState.notify(expectedResult)
                val result = awaitItem()
                assertEquals(expectedResult, result.tunnelRealState)
            }
        }

    @Test
    fun testTunnelUiStateUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedResult = TunnelState.Connected(mockk(), mockk())

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockk())
                relaySlot.captured.invoke(mockk(), mockk())
                eventNotifierTunnelUiState.notify(expectedResult)
                val result = awaitItem()
                assertEquals(expectedResult, result.tunnelUiState)
            }
        }

    @Test
    fun testAppVersionInfoUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedCurrentVersion = "1.0"
            val expectedUpgradeVersion = "2.0"
            val expectedIsOutdated = false
            val expectedIsSupported = false
            val expectedResult =
                VersionInfo(
                    currentVersion = expectedCurrentVersion,
                    upgradeVersion = expectedUpgradeVersion,
                    isOutdated = expectedIsOutdated,
                    isSupported = expectedIsSupported
                )

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockk())
                relaySlot.captured.invoke(mockk(), mockk())
                versionInfo.value = expectedResult
                val result = awaitItem()
                assertEquals(expectedResult, result.versionInfo)
            }
        }

    @Test
    fun testRelayItemUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedName = "Name"
            val expectedCode = "Code"
            val expectedExpanded = false
            val expectedCities = emptyList<RelayCity>()
            val expectedResult =
                RelayCountry(
                    name = expectedName,
                    code = expectedCode,
                    expanded = expectedExpanded,
                    cities = expectedCities
                )

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(mockk())
                relaySlot.captured.invoke(mockk(), expectedResult)
                val result = awaitItem()
                assertEquals(expectedResult, result.relayLocation)
            }
        }

    @Test
    fun testLocationUpdate() =
        runTest(testCoroutineRule.testDispatcher) {
            val expectedCountry = "Sweden"
            val expectedCity = "Gothenburg"
            val expectedHostname = "Host"
            val expectedIpv4: InetAddress = mockk()
            val expectedIpv6: InetAddress = mockk()
            val expectedResult =
                GeoIpLocation(
                    ipv4 = expectedIpv4,
                    ipv6 = expectedIpv6,
                    country = expectedCountry,
                    city = expectedCity,
                    hostname = expectedHostname
                )

            viewModel.uiState.test {
                assertEquals(ConnectUiState.INITIAL, awaitItem())
                serviceConnectionState.value =
                    ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
                locationSlot.captured.invoke(expectedResult)
                relaySlot.captured.invoke(mockk(), mockk())
                val result = awaitItem()
                assertEquals(expectedResult, result.location)
            }
        }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
    }
}
