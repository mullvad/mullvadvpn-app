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
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_COUNTRIES
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_ITEM_CUSTOM_LISTS
import net.mullvad.mullvadvpn.compose.screen.location.SelectLocationScreen
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.performLongClick
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
            setContentWithTheme { SelectLocationScreen(state = SelectLocationUiState.Loading) }

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
                            searchTerm = "",
                            filterChips = emptyList(),
                            relayListItems =
                                DUMMY_RELAY_COUNTRIES.map {
                                    RelayListItem.GeoLocationItem(item = it)
                                },
                            customLists = emptyList(),
                        )
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
    fun testSearchInput() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = "",
                            filterChips = emptyList(),
                            relayListItems = emptyList(),
                            customLists = emptyList(),
                        ),
                    onSearchTermInput = mockedSearchTermInput,
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
                            searchTerm = mockSearchString,
                            filterChips = emptyList(),
                            relayListItems =
                                listOf(RelayListItem.LocationsEmptyText(mockSearchString)),
                            customLists = emptyList(),
                        ),
                    onSearchTermInput = mockedSearchTermInput,
                )
            }

            // Assert
            onNodeWithText("No result for $mockSearchString.", substring = true).assertExists()
            onNodeWithText("Try a different search", substring = true).assertExists()
        }

    @Test
    fun customListFooterShouldShowEmptyTextWhenNoCustomList() =
        composeExtension.use {
            // Arrange
            val mockSearchString = ""
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = mockSearchString,
                            filterChips = emptyList(),
                            relayListItems = listOf(RelayListItem.CustomListFooter(false)),
                            customLists = emptyList(),
                        )
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
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = mockSearchString,
                            filterChips = emptyList(),
                            relayListItems = emptyList(),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                        )
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
            val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = "",
                            filterChips = emptyList(),
                            relayListItems = listOf(RelayListItem.CustomListItem(customList)),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                        ),
                    onSelectRelay = mockedOnSelectRelay,
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
            val customList = DUMMY_RELAY_ITEM_CUSTOM_LISTS[0]
            val mockedOnSelectRelay: (RelayItem) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                SelectLocationScreen(
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = "",
                            filterChips = emptyList(),
                            relayListItems =
                                listOf(RelayListItem.CustomListItem(item = customList)),
                            customLists = DUMMY_RELAY_ITEM_CUSTOM_LISTS,
                        ),
                    onSelectRelay = mockedOnSelectRelay,
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
                    state =
                        SelectLocationUiState.Content(
                            searchTerm = "",
                            filterChips = emptyList(),
                            relayListItems = listOf(RelayListItem.GeoLocationItem(relayItem)),
                            customLists = emptyList(),
                        ),
                    onSelectRelay = mockedOnSelectRelay,
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
