package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class SelectLocationViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockRelayListFilterUseCase: RelayListFilterUseCase = mockk(relaxed = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var viewModel: SelectLocationViewModel
    private val relayListWithSelectionFlow = MutableStateFlow(RelayList(emptyList(), null))
    private val mockRelayListUseCase: RelayListUseCase = mockk()
    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any())
    private val selectedProvider = MutableStateFlow<Constraint<Providers>>(Constraint.Any())
    private val allProvider = MutableStateFlow<List<Provider>>(emptyList())

    @Before
    fun setup() {

        every { mockRelayListFilterUseCase.selectedOwnership() } returns selectedOwnership
        every { mockRelayListFilterUseCase.selectedProviders()} returns selectedProvider
        every { mockRelayListFilterUseCase.availableProviders() } returns  allProvider
        every { mockRelayListUseCase.relayListWithSelection() } returns relayListWithSelectionFlow

        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)
        mockkStatic(RELAY_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                mockServiceConnectionManager,
                mockRelayListUseCase,
                mockRelayListFilterUseCase,
            )
    }

    @After
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun testInitialState() = runTest {
        assertEquals(SelectLocationUiState.Loading, viewModel.uiState.value)
    }

    @Test
    fun testUpdateLocations() = runTest {
        // Arrange
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedRelay: RelayItem = mockk()
        every { mockCountries.filterOnSearchTerm(any(), selectedRelay) } returns mockCountries
        relayListWithSelectionFlow.value = RelayList(mockCountries, selectedRelay)

        // Act, Assert
        viewModel.uiState.test {
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
        relayListWithSelectionFlow.value = RelayList(mockCountries, selectedRelay)

        // Act, Assert
        viewModel.uiState.test {
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
        every { mockServiceConnectionManager.connectionProxy() } returns connectionProxyMock
        every { mockRelayListUseCase.updateSelectedRelayLocation(mockLocation) } returns Unit

        // Act, Assert
        viewModel.uiCloseAction.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(Unit, awaitItem())
            verify {
                connectionProxyMock.connect()
                mockRelayListUseCase.updateSelectedRelayLocation(mockLocation)
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
        relayListWithSelectionFlow.value = RelayList(mockRelayList, selectedRelay)

        // Act, Assert
        viewModel.uiState.test {
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
        relayListWithSelectionFlow.value = RelayList(mockRelayList, selectedRelay)

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.ShowData>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.ShowData>(actualState)
            assertEquals(mockSearchString, actualState.searchTerm)
        }
    }

    @Test
    fun testRemoveOwnerFilter() = runTest {
        // Arrange
        val mockSelectedProviders: Constraint<Providers> = mockk()
        every { mockRelayListFilterUseCase.selectedProviders() } returns
            MutableStateFlow(
                mockSelectedProviders,
            )

        // Act
        viewModel.removeOwnerFilter()
        // Assert
        verify {
            mockRelayListFilterUseCase.updateOwnershipAndProviderFilter(
                any<Constraint.Any<Ownership>>(),
                mockSelectedProviders,
            )
        }
    }

    @Test
    fun testRemoveProviderFilter() = runTest {
        // Arrange
        val mockSelectedOwnership: Constraint<Ownership> = mockk()
        every { mockRelayListFilterUseCase.selectedOwnership() } returns
            MutableStateFlow(
                mockSelectedOwnership,
            )

        // Act
        viewModel.removeProviderFilter()
        // Assert
        verify {
            mockRelayListFilterUseCase.updateOwnershipAndProviderFilter(
                mockSelectedOwnership,
                any<Constraint.Any<Providers>>(),
            )
        }
    }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
    }
}
