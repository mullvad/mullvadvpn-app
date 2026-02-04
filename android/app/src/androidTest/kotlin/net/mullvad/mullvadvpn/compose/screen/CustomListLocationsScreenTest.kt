package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DUMMY_RELAY_COUNTRIES
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsData
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.util.DisableSoftKeyboard
import net.mullvad.mullvadvpn.core.Lce
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.CheckableRelayListItem
import net.mullvad.mullvadvpn.lib.ui.tag.CIRCULAR_PROGRESS_INDICATOR_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SAVE_BUTTON_TEST_TAG
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class CustomListLocationsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: CustomListLocationsUiState,
        onSearchTermInput: (String) -> Unit = {},
        onSaveClick: () -> Unit = {},
        onRelaySelectionClick: (RelayItem.Location, selected: Boolean) -> Unit = { _, _ -> },
        onExpand: (RelayItem.Location, selected: Boolean) -> Unit = { _, _ -> },
        onBackClick: () -> Unit = {},
        disableKeyboard: Boolean = false,
    ) {
        setContentWithTheme {
            DisableSoftKeyboard(disable = disableKeyboard) {
                CustomListLocationsScreen(
                    state = state,
                    onSearchTermInput = onSearchTermInput,
                    onSaveClick = onSaveClick,
                    onRelaySelectionClick = onRelaySelectionClick,
                    onExpand = onExpand,
                    onBackClick = onBackClick,
                )
            }
        }
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            initScreen(
                state = CustomListLocationsUiState(newList = false, content = Lce.Loading(Unit))
            )

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG).assertExists()
        }

    @Test
    fun givenNewListTrueShouldShowAddLocations() =
        composeExtension.use {
            // Arrange
            val newList = true
            initScreen(
                state = CustomListLocationsUiState(newList = newList, content = Lce.Loading(Unit))
            )

            // Assert
            onNodeWithText(ADD_LOCATIONS_TEXT).assertExists()
        }

    @Test
    fun givenNewListFalseShouldShowEditLocations() =
        composeExtension.use {
            // Arrange
            val newList = false
            initScreen(
                state = CustomListLocationsUiState(newList = newList, content = Lce.Loading(Unit))
            )

            // Assert
            onNodeWithText(EDIT_LOCATIONS_TEXT).assertExists()
        }

    @Test
    fun givenListOfAvailableLocationsShouldShowThem() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    locations =
                                        listOf(
                                            CheckableRelayListItem(
                                                DUMMY_RELAY_COUNTRIES[0],
                                                checked = true,
                                            ),
                                            CheckableRelayListItem(
                                                DUMMY_RELAY_COUNTRIES[1],
                                                checked = false,
                                            ),
                                        ),
                                    searchTerm = "",
                                    saveEnabled = false,
                                    hasUnsavedChanges = false,
                                )
                            ),
                    )
            )

            // Assert
            onNodeWithText("Relay Country 1").assertExists()
            onNodeWithText("Relay Country 2").assertExists()
        }

    @Test
    fun whenClickingOnRelayShouldCallOnSelectForThatRelay() =
        composeExtension.use {
            // Arrange
            val selectedCountry = DUMMY_RELAY_COUNTRIES[0]
            val mockedOnRelaySelectionClicked: (RelayItem, Boolean) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    locations =
                                        listOf(
                                            CheckableRelayListItem(selectedCountry, checked = true)
                                        ),
                                    searchTerm = "",
                                    saveEnabled = false,
                                    hasUnsavedChanges = false,
                                )
                            ),
                    ),
                onRelaySelectionClick = mockedOnRelaySelectionClicked,
            )

            // Act
            onNodeWithText(selectedCountry.name).performClick()

            // Assert
            verify { mockedOnRelaySelectionClicked(selectedCountry, false) }
        }

    @Test
    fun whenSearchInputIsUpdatedShouldCallOnSearchTermInput() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    locations = emptyList(),
                                    searchTerm = "",
                                    saveEnabled = false,
                                    hasUnsavedChanges = false,
                                )
                            ),
                    ),
                onSearchTermInput = mockedSearchTermInput,
                // This is required to avoid a crash due to the keyboard trying to open at the same
                // time as the test makes input
                disableKeyboard = true,
            )
            val mockSearchString = "SEARCH"

            // Act
            onNodeWithText(SEARCH_PLACEHOLDER).performTextInput(mockSearchString)

            // Assert
            verify { mockedSearchTermInput.invoke(mockSearchString) }
        }

    @Test
    fun whenSearchResultNotFoundShouldShowSearchNotFoundText() =
        composeExtension.use {
            // Arrange
            val mockedSearchTermInput: (String) -> Unit = mockk(relaxed = true)
            val mockSearchString = "SEARCH"
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    searchTerm = mockSearchString,
                                    saveEnabled = false,
                                    hasUnsavedChanges = false,
                                    locations = emptyList(),
                                )
                            ),
                    ),
                onSearchTermInput = mockedSearchTermInput,
            )

            // Assert
            onNodeWithText(EMPTY_SEARCH.format(mockSearchString)).assertExists()
        }

    @Test
    fun whenRelayListIsEmptyShouldShowNoRelaysText() =
        composeExtension.use {
            // Arrange
            initScreen(
                state = CustomListLocationsUiState(newList = false, content = Lce.Error(Unit))
            )

            // Assert
            onNodeWithText(NO_MATCHING_SERVERS_FOUNDS).assertExists()
        }

    @Test
    fun givenSaveIsEnabledWhenSaveClickedShouldCallOnSaveClick() =
        composeExtension.use {
            // Arrange
            val mockOnSaveClick: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    locations = emptyList(),
                                    saveEnabled = true,
                                    hasUnsavedChanges = true,
                                    searchTerm = "",
                                )
                            ),
                    ),
                onSaveClick = mockOnSaveClick,
            )

            // Act
            onNodeWithTag(SAVE_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockOnSaveClick() }
        }

    @Test
    fun givenSaveIsDisabledWhenSaveClickedShouldNotCallOnSaveClick() =
        composeExtension.use {
            // Arrange
            val mockOnSaveClick: () -> Unit = mockk(relaxed = true)
            initScreen(
                state =
                    CustomListLocationsUiState(
                        newList = false,
                        content =
                            Lce.Content(
                                CustomListLocationsData(
                                    locations = emptyList(),
                                    saveEnabled = false,
                                    hasUnsavedChanges = false,
                                    searchTerm = "",
                                )
                            ),
                    ),
                onSaveClick = mockOnSaveClick,
            )

            // Act
            onNodeWithTag(SAVE_BUTTON_TEST_TAG).performClick()

            // Assert
            verify(exactly = 0) { mockOnSaveClick() }
        }

    companion object {
        const val ADD_LOCATIONS_TEXT = "Add locations"
        const val EDIT_LOCATIONS_TEXT = "Edit locations"
        const val SEARCH_PLACEHOLDER = "Search for..."
        const val EMPTY_SEARCH = "No result for \"%s\", please try a different search"
        const val NO_LOCATIONS_FOUND_TEXT = "No locations found"
        const val NO_MATCHING_SERVERS_FOUNDS = "No matching servers found."
    }
}
