package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
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
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.allChildren
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class CustomListLocationsViewModelTest {
    private val mockRelayListUseCase: RelayListUseCase = mockk()
    private val mockCustomListUseCase: CustomListActionUseCase = mockk()

    private val relayListFlow = MutableStateFlow<List<RelayItem.Country>>(emptyList())
    private val customListFlow = MutableStateFlow<List<RelayItem.CustomList>>(emptyList())

    @BeforeEach
    fun setup() {
        every { mockRelayListUseCase.relayList() } returns relayListFlow
        every { mockRelayListUseCase.customLists() } returns customListFlow
    }

    @Test
    fun `given new list false state should return new list false`() = runTest {
        // Arrange
        val newList = false
        val viewModel = createViewModel("id", newList)

        // Act, Assert
        viewModel.uiState.test { assertEquals(newList, awaitItem().newList) }
    }

    @Test
    fun `when selected locations is not null and relay countries is not empty should return ui state content`() =
        runTest {
            // Arrange
            val expectedList = DUMMY_COUNTRIES
            val customListId = "id"
            val customListName = "name"
            val customList: RelayItem.CustomList = mockk {
                every { id } returns customListId
                every { name } returns customListName
                every { locations } returns emptyList()
            }
            customListFlow.value = listOf(customList)
            val expectedState =
                CustomListLocationsUiState.Content.Data(
                    newList = true,
                    availableLocations = expectedList
                )
            val viewModel = createViewModel(customListId, true)
            relayListFlow.value = expectedList

            // Act, Assert
            viewModel.uiState.test { assertEquals(expectedState, awaitItem()) }
        }

    @Test
    fun `when selecting parent should select children`() = runTest {
        // Arrange
        val expectedList = DUMMY_COUNTRIES
        val customListId = "id"
        val customListName = "name"
        val customList: RelayItem.CustomList = mockk {
            every { id } returns customListId
            every { name } returns customListName
            every { locations } returns emptyList()
        }
        customListFlow.value = listOf(customList)
        val expectedSelection =
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.allChildren() }).toSet()
        val viewModel = createViewModel(customListId, true)
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
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.allChildren() }).toSet()
        val customListId = "id"
        val customListName = "name"
        val customList: RelayItem.CustomList = mockk {
            every { id } returns customListId
            every { name } returns customListName
            every { locations } returns initialSelection.toList()
        }
        customListFlow.value = listOf(customList)
        val expectedSelection = emptySet<RelayItem>()
        val viewModel = createViewModel(customListId, true)
        relayListFlow.value = expectedList

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
            (DUMMY_COUNTRIES + DUMMY_COUNTRIES.flatMap { it.allChildren() }).toSet()
        val customListId = "id"
        val customListName = "name"
        val customList: RelayItem.CustomList = mockk {
            every { id } returns customListId
            every { name } returns customListName
            every { locations } returns initialSelection.toList()
        }
        customListFlow.value = listOf(customList)
        val expectedSelection = emptySet<RelayItem>()
        val viewModel = createViewModel(customListId, true)
        relayListFlow.value = expectedList

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
        val customListId = "id"
        val customListName = "name"
        val customList: RelayItem.CustomList = mockk {
            every { id } returns customListId
            every { name } returns customListName
            every { locations } returns emptyList()
        }
        customListFlow.value = listOf(customList)
        val expectedSelection = DUMMY_COUNTRIES[0].cities[0].relays.toSet()
        val viewModel = createViewModel(customListId, true)
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
            val customListId = "1"
            val customListName = "name"
            val newList = true
            val expectedResult: CustomListResult.LocationsChanged = mockk()
            val customList: RelayItem.CustomList = mockk {
                every { id } returns customListId
                every { name } returns customListName
                every { locations } returns DUMMY_COUNTRIES
            }
            customListFlow.value = listOf(customList)
            coEvery {
                mockCustomListUseCase.performAction(any<CustomListAction.UpdateLocations>())
            } returns Result.success(expectedResult)
            val viewModel = createViewModel(customListId, newList)

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
            val customListId = "1"
            val customListName = "name"
            val newList = false
            val expectedResult: CustomListResult.LocationsChanged = mockk()
            val customList: RelayItem.CustomList = mockk {
                every { id } returns customListId
                every { name } returns customListName
                every { locations } returns DUMMY_COUNTRIES
            }
            customListFlow.value = listOf(customList)
            coEvery {
                mockCustomListUseCase.performAction(any<CustomListAction.UpdateLocations>())
            } returns Result.success(expectedResult)
            val viewModel = createViewModel(customListId, newList)

            // Act, Assert
            viewModel.uiSideEffect.test {
                viewModel.save()
                val sideEffect = awaitItem()
                assertIs<CustomListLocationsSideEffect.ReturnWithResult>(sideEffect)
                assertEquals(expectedResult, sideEffect.result)
            }
        }

    private fun createViewModel(customListId: String, newList: Boolean) =
        CustomListLocationsViewModel(
            customListId = customListId,
            newList = newList,
            relayListUseCase = mockRelayListUseCase,
            customListActionUseCase = mockCustomListUseCase
        )

    companion object {
        private val DUMMY_COUNTRIES =
            listOf(
                RelayItem.Country(
                    name = "Sweden",
                    code = "SE",
                    expanded = false,
                    cities =
                        listOf(
                            RelayItem.City(
                                name = "Gothenburg",
                                code = "GBG",
                                expanded = false,
                                location = GeographicLocationConstraint.City("SE", "GBG"),
                                relays =
                                    listOf(
                                        RelayItem.Relay(
                                            name = "gbg-1",
                                            locationName = "GBG gbg-1",
                                            active = true,
                                            location =
                                                GeographicLocationConstraint.Hostname(
                                                    "SE",
                                                    "GBG",
                                                    "gbg-1"
                                                )
                                        )
                                    )
                            )
                        )
                )
            )
    }
}
