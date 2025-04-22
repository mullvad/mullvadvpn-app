package net.mullvad.mullvadvpn.viewmodel.location

import app.cash.turbine.test
import com.ramcosta.composedestinations.generated.navargs.toSavedStateHandle
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.screen.location.SearchLocationNavArgs
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SearchLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SearchLocationViewModelTest {

    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockFilteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk()
    private val mockCustomListsRepository: CustomListsRepository = mockk()
    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockFilterChipUseCase: FilterChipUseCase = mockk()
    private val mockFilteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase = mockk()
    private val mockSelectedLocationUseCase: SelectedLocationUseCase = mockk()
    private val mockCustomListsRelayItemUseCase: CustomListsRelayItemUseCase = mockk()

    private val filteredRelayList = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val selectedLocation =
        MutableStateFlow<RelayItemSelection>(RelayItemSelection.Single(Constraint.Any))
    private val filteredCustomListRelayItems =
        MutableStateFlow<List<RelayItem.CustomList>>(emptyList())
    private val customListRelayItems = MutableStateFlow<List<RelayItem.CustomList>>(emptyList())
    private val filterChips = MutableStateFlow<List<FilterChip>>(emptyList())
    private val wireguardConstraints = MutableStateFlow<WireguardConstraints>(mockk(relaxed = true))

    private lateinit var viewModel: SearchLocationViewModel

    @BeforeEach
    fun setup() {
        every { mockFilteredRelayListUseCase(any()) } returns filteredRelayList
        every { mockSelectedLocationUseCase() } returns selectedLocation
        every { mockFilteredCustomListRelayItemsUseCase(any()) } returns
            filteredCustomListRelayItems
        every { mockCustomListsRelayItemUseCase() } returns customListRelayItems
        every { mockFilterChipUseCase(any()) } returns filterChips
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            wireguardConstraints

        viewModel =
            SearchLocationViewModel(
                wireguardConstraintsRepository = mockWireguardConstraintsRepository,
                relayListRepository = mockRelayListRepository,
                filteredRelayListUseCase = mockFilteredRelayListUseCase,
                customListActionUseCase = mockCustomListActionUseCase,
                customListsRepository = mockCustomListsRepository,
                relayListFilterRepository = mockRelayListFilterRepository,
                filterChipUseCase = mockFilterChipUseCase,
                filteredCustomListRelayItemsUseCase = mockFilteredCustomListRelayItemsUseCase,
                selectedLocationUseCase = mockSelectedLocationUseCase,
                customListsRelayItemUseCase = mockCustomListsRelayItemUseCase,
                savedStateHandle =
                    SearchLocationNavArgs(relayListType = RelayListType.ENTRY).toSavedStateHandle(),
            )
    }

    @Test
    fun `on onSearchTermInput call uiState should emit with filtered countries`() = runTest {
        // Arrange
        val mockSearchString = "got"
        filteredRelayList.value = testCountries

        // Act, Assert
        viewModel.uiState.test() {
            // Wait for first data
            awaitItem()

            // Update search string
            viewModel.onSearchInputUpdated(mockSearchString)

            // We get some unnecessary emissions for now
            awaitItem()

            val actualState = awaitItem()
            assertIs<SearchLocationUiState.Content>(actualState)
            assertTrue(
                actualState.relayListItems.filterIsInstance<RelayListItem.GeoLocationItem>().any {
                    it.item is RelayItem.Location.City && it.item.name == "Gothenburg"
                }
            )
        }
    }

    @Test
    fun `when onSearchTermInput returns empty result uiState should return empty list`() = runTest {
        // Arrange
        filteredRelayList.value = testCountries
        val mockSearchString = "SEARCH"

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            awaitItem()

            // Update search string
            viewModel.onSearchInputUpdated(mockSearchString)

            // We get some unnecessary emissions for now
            awaitItem()

            // Assert
            val actualState = awaitItem()
            assertIs<SearchLocationUiState.Content>(actualState)
            assertLists(
                listOf(RelayListItem.LocationsEmptyText(mockSearchString, true)),
                actualState.relayListItems,
            )
        }
    }

    companion object {
        private val testCountries =
            listOf(
                RelayItem.Location.Country(
                    id = GeoLocationId.Country("se"),
                    "Sweden",
                    listOf(
                        RelayItem.Location.City(
                            id = GeoLocationId.City(GeoLocationId.Country("se"), "got"),
                            "Gothenburg",
                            emptyList(),
                        )
                    ),
                ),
                RelayItem.Location.Country(id = GeoLocationId.Country("no"), "Norway", emptyList()),
            )
    }
}
