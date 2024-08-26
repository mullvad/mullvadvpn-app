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
import kotlin.test.assertTrue
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.common.test.assertLists
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SelectLocationViewModelTest {

    private val mockRelayListFilterRepository: RelayListFilterRepository = mockk()
    private val mockAvailableProvidersUseCase: AvailableProvidersUseCase = mockk(relaxed = true)
    private val mockCustomListActionUseCase: CustomListActionUseCase = mockk(relaxed = true)
    private val mockFilteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase = mockk()
    private val mockFilteredRelayListUseCase: FilteredRelayListUseCase = mockk()
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockCustomListsRepository: CustomListsRepository = mockk()
    private val mockCustomListsRelayItemUseCase: CustomListsRelayItemUseCase = mockk()

    private val mockSettingsRepository: SettingsRepository = mockk()
    private val settingsFlow = MutableStateFlow(mockk<Settings>(relaxed = true))

    private lateinit var viewModel: SelectLocationViewModel

    private val allProviders = MutableStateFlow<List<Provider>>(emptyList())
    private val selectedOwnership = MutableStateFlow<Constraint<Ownership>>(Constraint.Any)
    private val selectedProviders = MutableStateFlow<Constraint<Providers>>(Constraint.Any)
    private val selectedRelayItemFlow = MutableStateFlow<Constraint<RelayItemId>>(Constraint.Any)
    private val filteredRelayList = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val filteredCustomRelayListItems =
        MutableStateFlow<List<RelayItem.CustomList>>(emptyList())
    private val customListsRelayItem = MutableStateFlow<List<RelayItem.CustomList>>(emptyList())

    @BeforeEach
    fun setup() {

        every { mockRelayListFilterRepository.selectedOwnership } returns selectedOwnership
        every { mockRelayListFilterRepository.selectedProviders } returns selectedProviders
        every { mockAvailableProvidersUseCase() } returns allProviders
        every { mockRelayListRepository.selectedLocation } returns selectedRelayItemFlow
        every { mockFilteredRelayListUseCase() } returns filteredRelayList
        every { mockFilteredCustomListRelayItemsUseCase() } returns filteredCustomRelayListItems
        every { mockCustomListsRelayItemUseCase() } returns customListsRelayItem
        every { mockSettingsRepository.settingsUpdates } returns settingsFlow

        mockkStatic(RELAY_LIST_EXTENSIONS)
        mockkStatic(RELAY_ITEM_EXTENSIONS)
        mockkStatic(CUSTOM_LIST_EXTENSIONS)
        viewModel =
            SelectLocationViewModel(
                relayListFilterRepository = mockRelayListFilterRepository,
                availableProvidersUseCase = mockAvailableProvidersUseCase,
                filteredCustomListRelayItemsUseCase = mockFilteredCustomListRelayItemsUseCase,
                customListActionUseCase = mockCustomListActionUseCase,
                filteredRelayListUseCase = mockFilteredRelayListUseCase,
                relayListRepository = mockRelayListRepository,
                customListsRepository = mockCustomListsRepository,
                customListsRelayItemUseCase = mockCustomListsRelayItemUseCase,
                settingsRepository = mockSettingsRepository,
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
    fun `given filteredRelayList emits update uiState should contain new update`() = runTest {
        // Arrange
        filteredRelayList.value = testCountries
        val selectedId = testCountries.first().id
        selectedRelayItemFlow.value = Constraint.Only(selectedId)

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
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
    fun `given relay is selected all relay items should not be selected`() = runTest {
        // Arrange
        filteredRelayList.value = testCountries
        selectedRelayItemFlow.value = Constraint.Any

        // Act, Assert
        viewModel.uiState.test {
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
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

    @Test
    fun `on selectRelay call uiSideEffect should emit CloseScreen and connect`() = runTest {
        // Arrange
        val mockRelayItem: RelayItem.Location.Country = mockk()
        val relayItemId: GeoLocationId.Country = mockk(relaxed = true)
        every { mockRelayItem.id } returns relayItemId
        coEvery { mockRelayListRepository.updateSelectedRelayLocation(relayItemId) } returns
            Unit.right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.selectRelay(mockRelayItem)
            // Await an empty item
            assertEquals(SelectLocationSideEffect.CloseScreen, awaitItem())
            coVerify { mockRelayListRepository.updateSelectedRelayLocation(relayItemId) }
        }
    }

    @Test
    fun `on onSearchTermInput call uiState should emit with filtered countries`() = runTest {
        // Arrange
        val mockSearchString = "got"
        filteredRelayList.value = testCountries
        selectedRelayItemFlow.value = Constraint.Any

        // Act, Assert
        viewModel.uiState.test {
            // Wait for first data
            assertIs<SelectLocationUiState.Content>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // We get some unnecessary emissions for now
            awaitItem()
            awaitItem()
            awaitItem()

            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
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
            assertIs<SelectLocationUiState.Content>(awaitItem())

            // Update search string
            viewModel.onSearchTermInput(mockSearchString)

            // We get some unnecessary emissions for now
            awaitItem()
            awaitItem()

            // Assert
            val actualState = awaitItem()
            assertIs<SelectLocationUiState.Content>(actualState)
            assertEquals(
                listOf(RelayListItem.LocationsEmptyText(mockSearchString)),
                actualState.relayListItems,
            )
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
        coVerify { mockCustomListActionUseCase(action) }
    }

    @Test
    fun `after adding a location to a list should emit location added side effect`() = runTest {
        // Arrange
        val customListId = CustomListId("1")
        val addedLocationsId = GeoLocationId.Country("se")
        val customListName = CustomListName.fromString("custom")
        val location: RelayItem.Location.Country = mockk {
            every { id } returns GeoLocationId.Country("se")
            every { name } returns "Sweden"
            every { descendants() } returns emptyList()
        }
        val customList =
            RelayItem.CustomList(
                customList =
                    CustomList(
                        id = CustomListId("1"),
                        name = customListName,
                        locations = emptyList(),
                    ),
                locations = emptyList(),
            )
        val expectedResult =
            CustomListActionResultData.Success.LocationAdded(
                customListName = customListName,
                locationName = location.name,
                undo = CustomListAction.UpdateLocations(id = customListId, locations = emptyList()),
            )

        coEvery { mockCustomListActionUseCase(any<CustomListAction.UpdateLocations>()) } returns
            LocationsChanged(
                    id = customListId,
                    name = customListName,
                    locations = listOf(addedLocationsId),
                    oldLocations = emptyList(),
                )
                .right()

        // Act, Assert
        viewModel.uiSideEffect.test {
            viewModel.addLocationToList(item = location, customList = customList)
            val sideEffect = awaitItem()
            assertIs<SelectLocationSideEffect.CustomListActionToast>(sideEffect)
            assertEquals(expectedResult, sideEffect.resultData)
        }
    }

    @Test
    fun `after removing a location from a list should emit location removed side effect`() =
        runTest {
            // Arrange
            val locationName = "Sweden"
            val customListId = CustomListId("1")
            val removedLocationsId = GeoLocationId.Country("se")
            val customListName = CustomListName.fromString("custom")
            val location: RelayItem.Location.Country = mockk {
                every { id } returns removedLocationsId
                every { name } returns locationName
                every { descendants() } returns emptyList()
            }
            val expectedResult =
                CustomListActionResultData.Success.LocationRemoved(
                    customListName = customListName,
                    locationName = locationName,
                    undo =
                        CustomListAction.UpdateLocations(
                            id = customListId,
                            locations = listOf(location.id),
                        ),
                )
            coEvery { mockCustomListActionUseCase(any<CustomListAction.UpdateLocations>()) } returns
                LocationsChanged(
                        id = customListId,
                        name = customListName,
                        locations = emptyList(),
                        oldLocations = listOf(removedLocationsId),
                    )
                    .right()
            coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns
                CustomList(
                        id = customListId,
                        name = customListName,
                        locations = listOf(removedLocationsId),
                    )
                    .right()

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.removeLocationFromList(item = location, customListId = customListId)
                val sideEffect = awaitItem()
                assertIs<SelectLocationSideEffect.CustomListActionToast>(sideEffect)
                assertEquals(expectedResult, sideEffect.resultData)
            }
        }

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
        private const val RELAY_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayListExtensionsKt"
        private const val RELAY_ITEM_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.RelayItemExtensionsKt"
        private const val CUSTOM_LIST_EXTENSIONS =
            "net.mullvad.mullvadvpn.relaylist.CustomListExtensionsKt"

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
