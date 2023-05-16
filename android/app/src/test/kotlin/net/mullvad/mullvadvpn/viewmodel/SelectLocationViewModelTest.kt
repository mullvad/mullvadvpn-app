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
import net.mullvad.mullvadvpn.assertLists
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.relayListListener
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SelectLocationViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var viewModel: SelectLocationViewModel

    // Service connections
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockRelayListListener: RelayListListener = mockk()

    // Captures
    private val relaySlot = slot<(RelayList, RelayItem?) -> Unit>()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    @Before
    fun setup() {
        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.relayListListener } returns mockRelayListListener

        every { mockRelayListListener.onRelayListChange = capture(relaySlot) } answers {}
        every { mockRelayListListener.onRelayListChange = null } answers {}

        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)

        viewModel = SelectLocationViewModel(mockServiceConnectionManager)
    }

    @After
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testInitialState() = runTest {
        viewModel.uiState.test { assertEquals(SelectLocationUiState.Loading, awaitItem()) }
    }

    @Test
    fun testUpdateLocations() = runTest {
        viewModel.uiState.test {
            val mockCountries = listOf<RelayCountry>(mockk(), mockk())
            val selectedRelay: RelayItem = mockk()
            val mockRelayList: RelayList = mockk()
            val expectedState = SelectLocationUiState.ShowData(mockCountries, selectedRelay)
            assertEquals(SelectLocationUiState.Loading, awaitItem())
            every { mockRelayList.countries } returns mockCountries
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockRelayList, selectedRelay)
            val actualState = awaitItem()
            if (actualState is SelectLocationUiState.ShowData) {
                assertLists(expectedState.countries, actualState.countries)
                assertEquals(expectedState.selectedRelay, actualState.selectedRelay)
            } else {
                throw Throwable("State is not ShowRelayList")
            }
        }
    }

    @Test
    fun testUpdateLocationsNoSelectedRelay() = runTest {
        viewModel.uiState.test {
            val mockCountries = listOf<RelayCountry>(mockk(), mockk())
            val selectedRelay: RelayItem? = null
            val mockRelayList: RelayList = mockk()
            val expectedState = SelectLocationUiState.ShowData(mockCountries, selectedRelay)
            assertEquals(SelectLocationUiState.Loading, awaitItem())
            every { mockRelayList.countries } returns mockCountries
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockRelayList, selectedRelay)
            val actualState = awaitItem()
            if (actualState is SelectLocationUiState.ShowData) {
                assertLists(expectedState.countries, actualState.countries)
                assertEquals(expectedState.selectedRelay, actualState.selectedRelay)
            } else {
                throw Throwable("State is not ShowRelayList")
            }
        }
    }

    @Test
    fun testSelectRelayAndClose() = runTest {
        viewModel.close.test {
            val mockRelayItem: RelayItem = mockk()
            val mockLocation: LocationConstraint.Country = mockk()
            val connectionProxyMock: ConnectionProxy = mockk()
            every { mockRelayItem.location } returns mockLocation
            every { mockRelayListListener.selectedRelayLocation = mockLocation } returns Unit
            every { mockServiceConnectionContainer.connectionProxy } returns connectionProxyMock
            every { connectionProxyMock.connect() } returns Unit
            every { mockLocation.countryCode } returns "MOCK-COUNTRY"
            every { mockServiceConnectionManager.relayListListener() } returns mockRelayListListener
            every { mockServiceConnectionManager.connectionProxy() } returns connectionProxyMock
            viewModel.selectRelay(mockRelayItem)
            val result = awaitItem() // Await an empty item
            assertEquals(Unit, result)
            verify {
                connectionProxyMock.connect()
                mockRelayListListener.selectedRelayLocation = mockLocation
            }
        }
    }

    companion object {
        private val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
    }
}
