package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
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
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SelectedLocationRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectLocationViewModelTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockAvailableProvidersUseCase: AvailableProvidersUseCase = mockk(relaxed = true)
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk(relaxed = true)
    private val mockCustomListsRelayItemUseCase: CustomListsRelayItemUseCase = mockk()
    private val mockSelectedLocationRelayItemUseCase: SelectedLocationRelayItemUseCase = mockk()
    private val mockFilteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val mockSelectedLocationRepository: SelectedLocationRepository = mockk()

    private lateinit var viewModel: SelectLocationViewModel

    private val allProviders = MutableStateFlow<List<Provider>>(emptyList())
    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)
    private val selectedRelayItemFlow = MutableStateFlow<RelayItem?>(null)
    private val filteredRelayList = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val customRelayListItems = MutableStateFlow<List<RelayItem.CustomList>>(emptyList())

    @BeforeEach
    fun setup() {

        every { mockRelayListFilterRepository.selectedOwnership } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedProviders } returns selectedProviders
        every { mockAvailableProvidersUseCase.availableProviders() } returns allProviders
        every { mockSelectedLocationRelayItemUseCase.selectedRelayItem() } returns
            selectedRelayItemFlow
        every { mockFilteredRelayListUseCase.filteredRelayList() } returns filteredRelayList
        every { mockCustomListsRelayItemUseCase.customListsRelayItems() } returns
            customRelayListItems

        mockkStatic(RELAY_LIST_EXTENSIONS)
        mockkStatic(RELAY_ITEM_EXTENSIONS)
        mockkStatic(CUSTOM_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                relayListFilterRepository = mockRelayListFilterRepository,
                availableProvidersUseCase = mockAvailableProvidersUseCase,
                customListsRelayItemUseCase = mockCustomListsRelayItemUseCase,
                selectedLocationRelayItemUseCase = mockSelectedLocationRelayItemUseCase,
                customListActionUseCase = mockCustomListActionUseCase,
                filteredRelayListUseCase = mockFilteredRelayListUseCase,
                selectedLocationRepository = mockSelectedLocationRepository
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
        val mockCountries = listOf<RelayItem.Location.Country>(mockk(), mockk())
        // val mockCustomList = listOf<RelayItem.CustomList>(mockk(relaxed = true))
        val selectedItem: RelayItem = mockk()
        every { mockCountries.filterOnSearchTerm(any(), selectedItem) } returns mockCountries
        filteredRelayList.value = mockCountries
        selectedRelayItemFlow.value = selectedItem

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
            val mockCountries = listOf<RelayItem.Location.Country>(mockk(), mockk())
            val selectedItem: RelayItem? = null
            every { mockCountries.filterOnSearchTerm(any(), selectedItem) } returns mockCountries
            filteredRelayList.value = mockCountries
            selectedRelayItemFlow.value = selectedItem

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
        val mockRelayItem: RelayItem.Location.Country = mockk()
        val relayItemId: GeoLocationId.Country = mockk(relaxed = true)
        every { mockRelayItem.id } returns relayItemId
        coEvery { mockSelectedLocationRepository.updateSelectedRelayLocation(relayItemId) } returns
            Unit.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(SelectLocationSideEffect.CloseScreen, awaitItem())
            coVerify { mockSelectedLocationRepository.updateSelectedRelayLocation(relayItemId) }
        }
    }

    @Test
    fun `on onSearchTermInput call uiState should emit with filtered countries`() = runTest {
        // Arrange
        val mockCustomList = listOf<RelayItem.CustomList>(mockk(relaxed = true))
        val mockCountries = listOf<RelayItem.Location.Country>(mockk(), mockk())
        val selectedItem: RelayItem? = null
        val mockRelayList: List<RelayItem.Location.Country> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedItem) } returns
            mockCountries
        every { mockCustomList.filterOnSearchTerm(mockSearchString) } returns mockCustomList
        filteredRelayList.value = mockRelayList
        selectedRelayItemFlow.value = selectedItem

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
        val mockCustomList = listOf<RelayItem.CustomList>(mockk(relaxed = true))
        val mockCountries = emptyList<RelayItem.Location.Country>()
        val selectedItem: RelayItem? = null
        val mockRelayList: List<RelayItem.Location.Country> = mockk(relaxed = true)
        val mockSearchString = "SEARCH"
        every { mockRelayList.filterOnSearchTerm(mockSearchString, selectedItem) } returns
            mockCountries
        every { mockCustomList.filterOnSearchTerm(mockSearchString) } returns mockCustomList

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
        every { mockRelayListFilterRepository.selectedProviders } returns
            MutableStateFlow(mockSelectedProviders)
        coEvery { mockRelayListFilterRepository.updateSelectedOwnership(Constraint.Any) } returns
            Unit.right()

        // Act
        viewModel.removeOwnerFilter()
        // Assert
        coVerify { mockRelayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    @Test
    fun `removeProviderFilter should invoke use case with Constraint Any Provider`() = runTest {
        // Arrange
        val mockSelectedOwnership: Constraint<Ownership> = mockk()
        every { mockRelayListFilterRepository.selectedOwnership } returns
            MutableStateFlow(mockSelectedOwnership)
        coEvery { mockRelayListFilterRepository.updateSelectedProviders(Constraint.Any) } returns
            Unit.right()

        // Act
        viewModel.removeProviderFilter()
        // Assert
        coVerify { mockRelayListFilterRepository.updateSelectedProviders(Constraint.Any) }
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
        val location: RelayItem.Location.Country = mockk {
            every { id } returns GeoLocationId.Country("se")
            every { descendants() } returns emptyList()
        }
        val customList =
            RelayItem.CustomList(
                id = CustomListId("1"),
                customListName = CustomListName.fromString("custom"),
                locations = emptyList(),
                expanded = false
            )
        coEvery {
            mockCustomListActionUseCase.performAction(any<CustomListAction.UpdateLocations>())
        } returns expectedResult.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.addLocationToList(item = location, customList = customList)
            val sideEffect = awaitItem()
            assertIs<SelectLocationSideEffect.LocationAddedToCustomList>(sideEffect)
            assertEquals(expectedResult, sideEffect.result)
        }
    }

    companion object {
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
        private const val RELAY_ITEM_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayItemExtensionsKt"
        private const val CUSTOM_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.CustomListExtensionsKt"
    }
}
