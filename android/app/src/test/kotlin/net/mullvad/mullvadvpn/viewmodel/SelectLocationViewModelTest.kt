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
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
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
    private val mockRelayListListener: RelayListListener = mockk(relaxUnitFun = true)

    // Captures
    private val relaySlot = slot<(List<RelayCountry>, RelayItem?) -> Unit>()

    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    @Before
    fun setup() {
        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.relayListListener } returns mockRelayListListener

        every { mockRelayListListener.onRelayCountriesChange = capture(relaySlot) } answers {}
        every { mockRelayListListener.onRelayCountriesChange = null } answers {}

        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)
        mockkStatic(RELAY_LIST_EXTENSIONS)

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
        // Arrange
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedRelay: RelayItem = mockk()
        every { mockCountries.filterOnSearchTerm(any(), selectedRelay) } returns mockCountries

        // Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockCountries, selectedRelay)

            assertEquals(SelectLocationUiState.Loading, awaitItem())
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.ShowData>(actualState)
            assertLists(mockCountries, actualState.countries)
            assertEquals(selectedRelay, actualState.selectedRelay)
        }
    }

    @Test
    fun testUpdateLocationsNoSelectedRelay() = runTest {
        // Arrange
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedRelay: RelayItem? = null
        every { mockCountries.filterOnSearchTerm(any(), selectedRelay) } returns mockCountries

        // Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockCountries, selectedRelay)

            assertEquals(SelectLocationUiState.Loading, awaitItem())
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.ShowData>(actualState)
            assertLists(mockCountries, actualState.countries)
            assertEquals(selectedRelay, actualState.selectedRelay)
        }
    }

    @Test
    fun testSelectRelayAndClose() = runTest {
        // Arrange
        val mockRelayItem: RelayItem = mockk()
        val mockLocation: GeographicLocationConstraint.Country = mockk(relaxed = true)
        val connectionProxyMock: ConnectionProxy = mockk(relaxUnitFun = true)
        every { mockRelayItem.location } returns mockLocation
        every { mockServiceConnectionManager.relayListListener() } returns mockRelayListListener
        every { mockServiceConnectionManager.connectionProxy() } returns connectionProxyMock

        // Act, Assert
        viewModel.uiCloseAction.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(Unit, awaitItem())
            verify {
                connectionProxyMock.connect()
                mockRelayListListener.updateSelectedRelayLocation(mockLocation)
            }
        }
    }

    @Test
    fun testFilterRelay() = runTest {
        // Arrange
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedRelay: RelayItem? = null
        val mockRelayList: List<RelayCountry> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedRelay) } returns
            mockCountries

        // Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockRelayList, selectedRelay)

            // Wait for loading
            assertEquals(SelectLocationUiState.Loading, awaitItem())
            // Wait for first data
            assertIs<SelectLocationUiState.ShowData>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.ShowData>(actualState)
            assertLists(mockCountries, actualState.countries)
            assertEquals(selectedRelay, actualState.selectedRelay)
        }
    }

    @Test
    fun testFilterNotFound() = runTest {
        // Arrange
        val mockCountries = emptyList<RelayCountry>()
        val selectedRelay: RelayItem? = null
        val mockRelayList: List<RelayCountry> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedRelay) } returns
            mockCountries

        // Act, Assert
        viewModel.uiState.test {
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            relaySlot.captured.invoke(mockRelayList, selectedRelay)

            // Wait for loading
            assertEquals(SelectLocationUiState.Loading, awaitItem())
            // Wait for first data
            assertIs<SelectLocationUiState.ShowData>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.NoSearchResultFound>(actualState)
            assertEquals(mockSearchString, actualState.searchTerm)
        }
    }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
