package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.runs
import io.mockk.unmockkAll
import io.mockk.verify
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.RelayListState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.relaylist.CustomRelayItemList
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.SelectedLocation
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectLocationViewModelTest {

    private val mockRelayListFilterUseCase: RelayListFilterUseCase = mockk(relaxed = true)
    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private lateinit var viewModel: SelectLocationViewModel
    private val relayListWithSelectionFlow =
        MutableStateFlow(RelayList(emptyList(), emptyList(), null))
    private val mockRelayListUseCase: RelayListUseCase = mockk()
    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any())
    private val selectedProvider = MutableStateFlow<Constraint<Providers>>(Constraint.Any())
    private val allProvider = MutableStateFlow<List<Provider>>(emptyList())

    @BeforeEach
    fun setup() {

        every { mockRelayListFilterUseCase.selectedOwnership() } returns selectedOwnership
        every { mockRelayListFilterUseCase.selectedProviders() } returns selectedProvider
        every { mockRelayListFilterUseCase.availableProviders() } returns allProvider
        every { mockRelayListUseCase.relayListWithSelection() } returns relayListWithSelectionFlow
        every { mockRelayListUseCase.fetchRelayList() } just runs

        mockkStatic(SERVICE_CONNECTION_MANAGER_EXTENSIONS)
        mockkStatic(RELAY_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                mockServiceConnectionManager,
                mockRelayListUseCase,
                mockRelayListFilterUseCase
            )
    }

    @AfterEach
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
        val mockCustomList = listOf<CustomRelayItemList>(mockk())
        val selectedLocation: SelectedLocation = mockk()
        every { mockCountries.filterOnSearchTerm(any(), selectedLocation) } returns mockCountries
        relayListWithSelectionFlow.value =
            RelayList(mockCustomList, mockCountries, selectedLocation)

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Data>(actualState)
            assertIs<RelayListState.RelayList>(actualState.relayListState)
            assertLists(
                mockCountries,
                (actualState.relayListState as RelayListState.RelayList).countries
            )
            assertEquals(
                selectedLocation,
                (actualState.relayListState as RelayListState.RelayList).selectedLocation
            )
        }
    }

    @Test
    fun testUpdateLocationsNoSelectedRelay() = runTest {
        // Arrange
        val mockCustomList = listOf<CustomRelayItemList>(mockk())
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedLocation: SelectedLocation? = null
        every { mockCountries.filterOnSearchTerm(any(), selectedLocation) } returns mockCountries
        relayListWithSelectionFlow.value =
            RelayList(mockCustomList, mockCountries, selectedLocation)

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Data>(actualState)
            assertIs<RelayListState.RelayList>(actualState.relayListState)
            assertLists(
                mockCountries,
                (actualState.relayListState as RelayListState.RelayList).countries
            )
            assertEquals(
                selectedLocation,
                (actualState.relayListState as RelayListState.RelayList).selectedLocation
            )
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
        viewModel.uiSideEffect.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(SelectLocationSideEffect.CloseScreen, awaitItem())
            verify {
                connectionProxyMock.connect()
                mockRelayListUseCase.updateSelectedRelayLocation(mockLocation)
            }
        }
    }

    @Test
    fun testFilterRelay() = runTest {
        // Arrange
        val mockCustomList = listOf<CustomRelayItemList>(mockk())
        val mockCountries = listOf<RelayCountry>(mockk(), mockk())
        val selectedLocation: SelectedLocation? = null
        val mockRelayList: List<RelayCountry> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedLocation) } returns
            mockCountries
        relayListWithSelectionFlow.value =
            RelayList(mockCustomList, mockRelayList, selectedLocation)

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.Data>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Data>(actualState)
            assertIs<RelayListState.RelayList>(actualState.relayListState)
            assertLists(
                mockCountries,
                (actualState.relayListState as RelayListState.RelayList).countries
            )
            assertEquals(
                selectedLocation,
                (actualState.relayListState as RelayListState.RelayList).selectedLocation
            )
        }
    }

    @Test
    fun testFilterNotFound() = runTest {
        // Arrange
        val mockCustomList = listOf<CustomRelayItemList>(mockk())
        val mockCountries = emptyList<RelayCountry>()
        val selectedLocation: SelectedLocation? = null
        val mockRelayList: List<RelayCountry> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedLocation) } returns
            mockCountries
        relayListWithSelectionFlow.value =
            RelayList(mockCustomList, mockRelayList, selectedLocation)

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.Data>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Data>(actualState)
            assertEquals(mockSearchString, actualState.searchTerm)
        }
    }

    @Test
    fun testRemoveOwnerFilter() = runTest {
        // Arrange
        val mockSelectedProviders: Constraint<Providers> = mockk()
        every { mockRelayListFilterUseCase.selectedProviders() } returns
            MutableStateFlow(mockSelectedProviders)

        // Act
        viewModel.removeOwnerFilter()
        // Assert
        verify {
            mockRelayListFilterUseCase.updateOwnershipAndProviderFilter(
                any<Constraint.Any<Ownership>>(),
                mockSelectedProviders
            )
        }
    }

    @Test
    fun testRemoveProviderFilter() = runTest {
        // Arrange
        val mockSelectedOwnership: Constraint<Ownership> = mockk()
        every { mockRelayListFilterUseCase.selectedOwnership() } returns
            MutableStateFlow(mockSelectedOwnership)

        // Act
        viewModel.removeProviderFilter()
        // Assert
        verify {
            mockRelayListFilterUseCase.updateOwnershipAndProviderFilter(
                mockSelectedOwnership,
                any<Constraint.Any<Providers>>()
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
