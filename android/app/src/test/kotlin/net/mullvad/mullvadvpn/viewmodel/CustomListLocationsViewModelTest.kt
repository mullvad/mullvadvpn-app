package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.Either
import arrow.core.right
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeoLocationId
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Provider
import net.mullvad.mullvadvpn.model.ProviderId
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListRelayItemsUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CustomListLocationsViewModelTest {
    private val mockRelayListRepository: RelayListRepository = mockk()
    private val mockCustomListUseCase: CustomListActionUseCase = mockk()
    private val mockCustomListRelayItemsUseCase: CustomListRelayItemsUseCase = mockk()
    private val mockCustomListsRepository: CustomListsRepository = mockk()

    private val relayListFlow = MutableStateFlow<List<RelayItem.Location.Country>>(emptyList())
    private val selectedLocationsFlow = MutableStateFlow<List<RelayItem.Location>>(emptyList())

    @BeforeEach
    fun setup() {
        every { mockRelayListRepository.relayList } returns relayListFlow
        every { mockCustomListRelayItemsUseCase.getRelayItemLocationsForCustomList(any()) } returns
            selectedLocationsFlow
    }

    @Test
    fun `given new list false state should return new list false`() = runTest {
        // Arrange
        val newList = false
        val customList =
            CustomList(
                id = CustomListId("id"),
                name = CustomListName.fromString("name"),
                locations = emptyList()
            )
        val viewModel =
            createViewModel(
                customList = customList.right(),
                customListId = customList.id,
                newList = newList
            )

        // Act, Assert
        viewModel.uiState.test { assertEquals(newList, awaitItem().newList) }
    }

    @Test
    fun `when selected locations is not null and relay countries is not empty should return ui state content`() =
        runTest {
            // Arrange
            val expectedList = DUMMY_COUNTRIES
            val customListId = CustomListId("id")
            val customListName = CustomListName.fromString("name")
            val customList =
                CustomList(id = customListId, name = customListName, locations = emptyList())
            val expectedState =
                CustomListLocationsUiState.Content.Data(
                    newList = true,
                    availableLocations = expectedList
                )
            val viewModel = createViewModel(customListId, customList = customList.right(), true)
            relayListFlow.value = expectedList

            // Act, Assert
            viewModel.uiState.test { assertEquals(expectedState, awaitItem()) }
        }

    @Test
    fun `when selecting parent should select children`() = runTest {
        // Arrange
        val expectedList = DUMMY_COUNTRIES
        val customListId = CustomListId("id")
        val customListName = CustomListName.fromString("name")
        val customList =
            CustomList(id = customListId, name = customListName, locations = emptyList())
        val expectedSelection =
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.descendants() }).toSet()
        val viewModel = createViewModel(customListId, customList.right(), true)
        relayListFlow.value = expectedList

        // Act, Assert
        viewModel.uiState.test {
            // Check no selected
            val firstState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(firstState)
            assertEquals(emptySet<RelayItem>(), firstState.selectedLocations)
            viewModel.onRelaySelectionClick(DUMMY_COUNTRIES[0], true)
            // Check all items selected
            val secondState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(secondState)
            assertEquals(expectedSelection, secondState.selectedLocations)
        }
    }

    @Test
    fun `when deselecting child should deselect parent`() = runTest {
        // Arrange
        val expectedList = DUMMY_COUNTRIES
        val initialSelection =
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.descendants() }).toSet()
        val customListId = CustomListId("id")
        val customListName = CustomListName.fromString("name")
        val customList =
            CustomList(id = customListId, name = customListName, locations = emptyList())
        val expectedSelection = emptySet<RelayItem>()
        relayListFlow.value = expectedList
        selectedLocationsFlow.value = initialSelection.toList()
        val viewModel = createViewModel(customListId, customList.right(), true)

        // Act, Assert
        viewModel.uiState.test {
            // Check initial selected
            val firstState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(firstState)
            assertEquals(initialSelection, firstState.selectedLocations)
            viewModel.onRelaySelectionClick(DUMMY_COUNTRIES[0].cities[0].relays[0], false)
            // Check all items selected
            val secondState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(secondState)
            assertEquals(expectedSelection, secondState.selectedLocations)
        }
    }

    @Test
    fun `when deselecting parent should deselect child`() = runTest {
        // Arrange
        val expectedList = DUMMY_COUNTRIES
        val initialSelection =
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.descendants() }).toSet()
        val customListId = CustomListId("id")
        val customListName = CustomListName.fromString("name")
        val customList =
            CustomList(id = customListId, name = customListName, locations = emptyList())
        val expectedSelection = emptySet<RelayItem>()
        relayListFlow.value = expectedList
        selectedLocationsFlow.value = initialSelection.toList()
        val viewModel = createViewModel(customListId, customList.right(), true)

        // Act, Assert
        viewModel.uiState.test {
            // Check initial selected
            val firstState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(firstState)
            assertEquals(initialSelection, firstState.selectedLocations)
            viewModel.onRelaySelectionClick(DUMMY_COUNTRIES[0], false)
            // Check all items selected
            val secondState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(secondState)
            assertEquals(expectedSelection, secondState.selectedLocations)
        }
    }

    @Test
    fun `when selecting child should not select parent`() = runTest {
        // Arrange
        val expectedList = DUMMY_COUNTRIES
        val customListId = CustomListId("id")
        val customListName = CustomListName.fromString("name")
        val customList =
            CustomList(id = customListId, name = customListName, locations = emptyList())
        val expectedSelection = DUMMY_COUNTRIES[0].cities[0].relays.toSet()
        val viewModel = createViewModel(customListId, customList = customList.right(), true)
        relayListFlow.value = expectedList

        // Act, Assert
        viewModel.uiState.test {
            // Check no selected
            val firstState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(firstState)
            assertEquals(emptySet<RelayItem>(), firstState.selectedLocations)
            viewModel.onRelaySelectionClick(DUMMY_COUNTRIES[0].cities[0].relays[0], true)
            // Check all items selected
            val secondState = awaitItem()
            assertIs<CustomListLocationsUiState.Content.Data>(secondState)
            assertEquals(expectedSelection, secondState.selectedLocations)
        }
    }

    @Test
    fun `given new list true when saving successfully should emit close screen side effect`() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val customListName = CustomListName.fromString("name")
            val newList = true
            val expectedResult: CustomListResult.LocationsChanged = mockk()
            val customList =
                CustomList(id = customListId, name = customListName, locations = emptyList())
            coEvery {
                mockCustomListUseCase.performAction(any<CustomListAction.UpdateLocations>())
            } returns expectedResult.right()
            val viewModel = createViewModel(customListId, customList = customList.right(), newList)

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save()
                val sideEffect = awaitItem()
                assertIs<CustomListLocationsSideEffect.CloseScreen>(sideEffect)
            }
        }

    @Test
    fun `given new list false when saving successfully should emit return with result side effect`() =
        runTest {
            // Arrange
            val customListId = CustomListId("1")
            val customListName = CustomListName.fromString("name")
            val newList = false
            val expectedResult: CustomListResult.LocationsChanged = mockk()
            val customList =
                CustomList(id = customListId, name = customListName, locations = emptyList())
            coEvery {
                mockCustomListUseCase.performAction(any<CustomListAction.UpdateLocations>())
            } returns expectedResult.right()
            val viewModel = createViewModel(customListId, customList = customList.right(), newList)

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save()
                val sideEffect = awaitItem()
                assertIs<CustomListLocationsSideEffect.ReturnWithResult>(sideEffect)
                assertEquals(expectedResult, sideEffect.result)
            }
        }

    private fun createViewModel(
        customListId: CustomListId,
        customList: Either<GetCustomListError, CustomList>,
        newList: Boolean
    ): CustomListLocationsViewModel {
        coEvery { mockCustomListsRepository.getCustomListById(customListId) } returns customList
        return CustomListLocationsViewModel(
            customListId = customListId,
            newList = newList,
            relayListRepository = mockRelayListRepository,
            customListRelayItemsUseCase = mockCustomListRelayItemsUseCase,
            customListActionUseCase = mockCustomListUseCase,
            customListsRepository = mockCustomListsRepository
        )
    }

    companion object {
        private val DUMMY_COUNTRIES =
            listOf(
                RelayItem.Location.Country(
                    name = "Sweden",
                    id = GeoLocationId.Country("SE"),
                    expanded = false,
                    cities =
                        listOf(
                            RelayItem.Location.City(
                                name = "Gothenburg",
                                expanded = false,
                                id = GeoLocationId.City(GeoLocationId.Country("SE"), "GBG"),
                                relays =
                                    listOf(
                                        RelayItem.Location.Relay(
                                            id =
                                                GeoLocationId.Hostname(
                                                    GeoLocationId.City(
                                                        GeoLocationId.Country("SE"),
                                                        "GBG"
                                                    ),
                                                    "gbg-1"
                                                ),
                                            active = true,
                                            provider =
                                                Provider(
                                                    ProviderId("Provider"),
                                                    ownership = Ownership.MullvadOwned
                                                )
                                        )
                                    )
                            )
                        )
                )
            )
    }
}
