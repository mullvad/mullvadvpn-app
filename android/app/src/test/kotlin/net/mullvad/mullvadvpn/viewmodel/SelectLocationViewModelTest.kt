package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.coVerify
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
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.relaylist.toLocationConstraint
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
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
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()
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
        mockkStatic(RELAY_ITEM_EXTENSIONS)
        mockkStatic(CUSTOM_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                mockServiceConnectionManager,
                mockRelayListUseCase,
                mockRelayListFilterUseCase,
                mockCustomListActionUseCase
            )
    }

    @AfterEach
    fun teardown() {
        viewModel.viewModelScope.coroutineContext.cancel()
        unmockkAll()
    }

    @Test
    fun `initial state should be loading`() = runTest {
        assertEquals(SelectLocationUiState.Loading, viewModel.uiState.value)
    }

    @Test
    fun `given relayListWithSelection emits update uiState should contain new update`() = runTest {
        // Arrange
        val mockCountries = listOf<RelayItem.Country>(mockk(), mockk())
        val mockCustomList = listOf<RelayItem.CustomList>(mockk())
        val selectedItem: RelayItem = mockk()
        every { mockCountries.filterOnSearchTerm(any(), selectedItem) } returns mockCountries
        relayListWithSelectionFlow.value = RelayList(mockCustomList, mockCountries, selectedItem)

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
            assertLists(mockCountries, actualState.countries)
            assertEquals(selectedItem, actualState.selectedItem)
        }
    }

    @Test
    fun `given relayListWithSelection emits update with no selections selectedItem should be null`() =
        runTest {
            // Arrange
            val mockCustomList = listOf<RelayItem.CustomList>(mockk())
            val mockCountries = listOf<RelayItem.Country>(mockk(), mockk())
            val selectedItem: RelayItem? = null
            every { mockCountries.filterOnSearchTerm(any(), selectedItem) } returns mockCountries
            relayListWithSelectionFlow.value =
                RelayList(mockCustomList, mockCountries, selectedItem)

            // Act, Assert
            viewModel.uiState.test {
                val actualState = awaitItem()
                assertIs<SelectLocationUiState.Content>(actualState)
                assertLists(mockCountries, actualState.countries)
                assertEquals(selectedItem, actualState.selectedItem)
            }
        }

    @Test
    fun `on selectRelay call uiSideEffect should emit CloseScreen and connect`() = runTest {
        // Arrange
        val mockRelayItem: RelayItem.Country = mockk()
        val mockLocation: GeographicLocationConstraint.Country = mockk(relaxed = true)
        val mockLocationConstraint: LocationConstraint = mockk()
        val connectionProxyMock: ConnectionProxy = mockk(relaxUnitFun = true)
        every { mockRelayItem.location } returns mockLocation
        every { mockServiceConnectionManager.connectionProxy() } returns connectionProxyMock
        every { mockRelayListUseCase.updateSelectedRelayLocation(mockLocationConstraint) } returns
            Unit
        every { mockRelayItem.toLocationConstraint() } returns mockLocationConstraint

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(SelectLocationSideEffect.CloseScreen, awaitItem())
            verify {
                connectionProxyMock.connect()
                mockRelayListUseCase.updateSelectedRelayLocation(mockLocationConstraint)
            }
        }
    }

    @Test
    fun `on onSearchTermInput call uiState should emit with filtered countries`() = runTest {
        // Arrange
        val mockCustomList = listOf<RelayItem.CustomList>(mockk())
        val mockCountries = listOf<RelayItem.Country>(mockk(), mockk())
        val selectedItem: RelayItem? = null
        val mockRelayList: List<RelayItem.Country> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedItem) } returns
            mockCountries
        every { mockCustomList.filterOnSearchTerm(mockSearchString) } returns mockCustomList
        relayListWithSelectionFlow.value = RelayList(mockCustomList, mockRelayList, selectedItem)

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.Content>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
            assertLists(mockCountries, actualState.countries)
            assertEquals(selectedItem, actualState.selectedItem)
        }
    }

    @Test
    fun `when onSearchTermInput returns empty result uiState should return empty list`() = runTest {
        // Arrange
        val mockCustomList = listOf<RelayItem.CustomList>(mockk())
        val mockCountries = emptyList<RelayItem.Country>()
        val selectedItem: RelayItem? = null
        val mockRelayList: List<RelayItem.Country> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedItem) } returns
            mockCountries
        every { mockCustomList.filterOnSearchTerm(mockSearchString) } returns mockCustomList
        relayListWithSelectionFlow.value = RelayList(mockCustomList, mockRelayList, selectedItem)

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.Content>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
            assertEquals(mockSearchString, actualState.searchTerm)
        }
    }

    @Test
    fun `removeOwnerFilter should invoke use case with Constraint Any Ownership`() = runTest {
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
    fun `removeProviderFilter should invoke use case with Constraint Any Provider`() = runTest {
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

    @Test
    fun `when perform action is called should call custom list use case`() {
        // Arrange
        val action: CustomListAction = mockk()

        // Act
        viewModel.performAction(action)

        // Assert
        coVerify { mockCustomListActionUseCase.performAction(action) }
    }

    @Test
    fun `after adding a location to a list should emit location added side effect`() = runTest {
        // Arrange
        val expectedResult: CustomListResult.LocationsChanged = mockk()
        val location: RelayItem = mockk { every { code } returns "code" }
        val customList: RelayItem.CustomList = mockk {
            every { id } returns "1"
            every { locations } returns emptyList()
        }
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.UpdateLocations>())
        } returns Result.success(expectedResult)

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.addLocationToList(item = location, customList = customList)
            val sideEffect = awaitItem()
            assertIs<SelectLocationSideEffect.LocationAddedToCustomList>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    companion object {
        private const val SERVICE_CONNECTION_MANAGER_EXTENSIONS =
            "net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManagerExtensionsKt"
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
        private const val RELAY_ITEM_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayItemExtensionsKt"
        private const val CUSTOM_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.CustomListExtensionsKt"
    }
}
