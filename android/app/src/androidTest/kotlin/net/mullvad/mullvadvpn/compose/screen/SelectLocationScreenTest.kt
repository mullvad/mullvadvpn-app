package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DUMMY_CUSTOM_LISTS
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_COUNTRIES
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.performLongClick
import net.mullvad.mullvadvpn.relaylist.RelayItem
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class SelectLocationScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    state = SelectLocationUiState.Loading,
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun testShowRelayListState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = DUMMY_RELAY_COUNTRIES,
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                )
            }

            // Assert
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertDoesNotExist()
            onNodeWithText("Relay host 1").assertDoesNotExist()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }

    @Test
    fun testShowRelayListStateSelected() =
        composeExtension.use {
            val updatedDummyList =
                DUMMY_RELAY_COUNTRIES.let {
                    val cities = it[0].cities.toMutableList()
                    val city = cities.removeAt(0)
                    cities.add(0, city.copy(expanded = true))

                    val mutableRelayList = it.toMutableList()
                    mutableRelayList[0] = it[0].copy(expanded = true, cities = cities.toList())
                    mutableRelayList
                }

            // Arrange
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = updatedDummyList,
                            selectedItem = updatedDummyList[0].cities[0].relays[0],
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                )
            }

            // Assert
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay City 1").assertExists()
            onNodeWithText("Relay host 1").assertExists()
            onNodeWithText("Relay Country 2").assertExists()
            onNodeWithText("Relay City 2").assertDoesNotExist()
            onNodeWithText("Relay host 2").assertDoesNotExist()
        }

    @Test
    fun testSearchInput() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }
            val mockSearchString = "SEARCH"

            // Act
            onNodeWithText("Search for...").performTextInput(mockSearchString)

            // Assert
            verify { mockedSearchTermInput.invoke(mockSearchString) }
        }

    @Test
    fun testSearchTermNotFound() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            val mockSearchString = "SEARCH"
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = mockSearchString
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }

            // Assert
            onNodeWithText("No result for $mockSearchString.", substring = true).assertExists()
            onNodeWithText("Try a different search", substring = true).assertExists()
        }

    @Test
    fun givenNoCustomListsAndSearchIsTermIsEmptyShouldShowCustomListsEmptyText() =
        composeExtension.use {
            // Arrange
            val mockSearchString = ""
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = mockSearchString
                        ),
                )
            }

            // Assert
            onNodeWithText(CUSTOM_LISTS_EMPTY_TEXT).assertExists()
        }

    @Test
    fun givenNoCustomListsAndSearchIsActiveShouldNotShowCustomListHeader() =
        composeExtension.use {
            // Arrange
            val mockSearchString = "SEARCH"
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Content(
                            customLists = DUMMY_CUSTOM_LISTS,
                            filteredCustomLists = emptyList(),
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = mockSearchString
                        ),
                )
            }

            // Assert
            onNodeWithText(CUSTOM_LISTS_EMPTY_TEXT).assertDoesNotExist()
            onNodeWithTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG).assertDoesNotExist()
        }

    @Test
    fun whenCustomListIsClickedShouldCallOnSelectRelay() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Content(
                            customLists = DUMMY_CUSTOM_LISTS,
                            filteredCustomLists = DUMMY_CUSTOM_LISTS,
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                    onSelectRelay = mockedOnSelectRelay
                )
            }

            // Act
            onNodeWithText(customList.name).performClick()

            // Assert
            verify { mockedOnSelectRelay(customList) }
        }

    @Test
    fun whenCustomListIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Content(
                            customLists = DUMMY_CUSTOM_LISTS,
                            filteredCustomLists = DUMMY_CUSTOM_LISTS,
                            countries = emptyList(),
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                    onSelectRelay = mockedOnSelectRelay
                )
            }

            // Act
            onNodeWithText(customList.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG)
        }

    @Test
    fun whenLocationIsLongClickedShouldShowBottomSheet() =
        composeExtension.use {
            // Arrange
            val relayItem = DUMMY_RELAY_COUNTRIES[0]
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    uiState =
                        SelectLocationUiState.Content(
                            customLists = emptyList(),
                            filteredCustomLists = emptyList(),
                            countries = DUMMY_RELAY_COUNTRIES,
                            selectedItem = null,
                            selectedOwnership = null,
                            selectedProvidersCount = 0,
                            searchTerm = ""
                        ),
                    onSelectRelay = mockedOnSelectRelay
                )
            }

            // Act
            onNodeWithText(relayItem.name).performLongClick()

            // Assert
            onNodeWithTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG)
        }

    companion object {
        private const val CUSTOM_LISTS_EMPTY_TEXT = "To create a custom list press the \"︙\""
    }
}
