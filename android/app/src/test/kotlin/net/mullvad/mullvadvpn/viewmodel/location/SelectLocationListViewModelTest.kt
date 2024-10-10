package net.mullvad.mullvadvpn.viewmodel.location

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectLocationListViewModelTest {

    private val mockFilteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val mockFilteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase = mockk()
    private val mockSelectedLocationUseCase: SelectedLocationUseCase = mockk()
    private val mockWireguardConstraintsRepository: WireguardConstraintsRepository = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockCustomListRelayItemsUseCase: CustomListsRelayItemUseCase = mockk()

    private val filteredRelayList = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val selectedLocationFlow = MutableStateFlow<RelayItemSelection>(mockk(relaxed = true))
    private val filteredCustomListRelayItems =
        MutableStateFlow<List<RelayItem.CustomList>>(emptyList())
    private val customListRelayItems = MutableStateFlow<List<RelayItem.CustomList>>(emptyList())

    private lateinit var viewModel: SelectLocationListViewModel

    @BeforeEach
    fun setUp() {
        // Used for initial selection
        every { mockRelayListRepository.selectedLocation } returns MutableStateFlow(Constraint.Any)
        every { mockWireguardConstraintsRepository.wireguardConstraints } returns
            MutableStateFlow(null)

        every { mockSelectedLocationUseCase() } returns selectedLocationFlow
        every { mockFilteredRelayListUseCase() } returns filteredRelayList
        every { mockFilteredCustomListRelayItemsUseCase() } returns filteredCustomListRelayItems
        every { mockCustomListRelayItemsUseCase() } returns customListRelayItems
    }

    @Test
    fun `initial state should be loading`() = runTest {
        // Arrange
        viewModel = createSelectLocationListViewModel(relayListType = RelayListType.ENTRY)

        // Assert
        assertEquals(SelectLocationListUiState.Loading, viewModel.uiState.value)
    }

    @Test
    fun `given filteredRelayList emits update uiState should contain new update`() = runTest {
        // Arrange
        viewModel = createSelectLocationListViewModel(RelayListType.EXIT)
        filteredRelayList.value = testCountries
        val selectedId = testCountries.first().id
        selectedLocationFlow.value = RelayItemSelection.Single(Constraint.Only(selectedId))

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationListUiState.Content>(actualState)
            assertLists(
                testCountries.map { it.id },
                actualState.relayListItems.mapNotNull { it.relayItemId() },
            )
            assertTrue(
                actualState.relayListItems
                    .filterIsInstance<RelayListItem.SelectableItem>()
                    .first { it.relayItemId() == selectedId }
                    .isSelected
            )
        }
    }

    @Test
    fun `given relay is not selected all relay items should not be selected`() = runTest {
        // Arrange
        viewModel = createSelectLocationListViewModel(RelayListType.EXIT)
        filteredRelayList.value = testCountries
        selectedLocationFlow.value = RelayItemSelection.Single(Constraint.Any)

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationListUiState.Content>(actualState)
            assertLists(
                testCountries.map { it.id },
                actualState.relayListItems.mapNotNull { it.relayItemId() },
            )
            assertTrue(
                actualState.relayListItems.filterIsInstance<RelayListItem.SelectableItem>().all {
                    !it.isSelected
                }
            )
        }
    }

    private fun createSelectLocationListViewModel(relayListType: RelayListType) =
        SelectLocationListViewModel(
            relayListType = relayListType,
            filteredRelayListUseCase = mockFilteredRelayListUseCase,
            filteredCustomListRelayItemsUseCase = mockFilteredCustomListRelayItemsUseCase,
            selectedLocationUseCase = mockSelectedLocationUseCase,
            wireguardConstraintsRepository = mockWireguardConstraintsRepository,
            relayListRepository = mockRelayListRepository,
            customListsRelayItemUseCase = mockCustomListRelayItemsUseCase,
        )

    private fun RelayListItem.relayItemId() =
        when (this) {
            is RelayListItem.CustomListFooter -> null
            RelayListItem.CustomListHeader -> null
            RelayListItem.LocationHeader -> null
            is RelayListItem.LocationsEmptyText -> null
            is RelayListItem.CustomListEntryItem -> item.id
            is RelayListItem.CustomListItem -> item.id
            is RelayListItem.GeoLocationItem -> item.id
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
