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
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.CustomListLocationsUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SAVE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.relaylist.RelayItem
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

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState = CustomListLocationsUiState.Loading(newList = false)
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun givenNewListTrueShouldShowAddLocations() =
        composeExtension.use {
            // Arrange
            val newList = true
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState = CustomListLocationsUiState.Loading(newList = newList)
                )
            }

            // Assert
            onNodeWithText(ADD_LOCATIONS_TEXT).assertExists()
        }

    @Test
    fun givenNewListFalseShouldShowEditLocations() =
        composeExtension.use {
            // Arrange
            val newList = false
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState = CustomListLocationsUiState.Loading(newList = newList)
                )
            }

            // Assert
            onNodeWithText(EDIT_LOCATIONS_TEXT).assertExists()
        }

    @Test
    fun givenListOfAvailableLocationsShouldShowThem() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Data(
                            availableLocations = DUMMY_RELAY_COUNTRIES,
                            selectedLocations = emptySet(),
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
    fun whenClickingOnRelayShouldCallOnSelectForThatRelay() =
        composeExtension.use {
            // Arrange
            val selectedCountry = DUMMY_RELAY_COUNTRIES[0]
            val mockedOnRelaySelectionClicked: (RelayItem, Boolean) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Data(
                            newList = false,
                            availableLocations = DUMMY_RELAY_COUNTRIES,
                            selectedLocations = setOf(selectedCountry)
                        ),
                    onRelaySelectionClick = mockedOnRelaySelectionClicked
                )
            }

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
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Data(
                            newList = false,
                            availableLocations = DUMMY_RELAY_COUNTRIES,
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }
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
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Empty(
                            newList = false,
                            searchTerm = mockSearchString
                        ),
                    onSearchTermInput = mockedSearchTermInput
                )
            }

            // Assert
            onNodeWithText(EMPTY_SEARCH_FIRST_ROW.format(mockSearchString), substring = true)
                .assertExists()
            onNodeWithText(EMPTY_SEARCH_SECOND_ROW, substring = true).assertExists()
        }

    @Test
    fun whenRelayListIsEmptyShouldShowNoRelaysText() =
        composeExtension.use {
            // Arrange
            val emptySearchString = ""
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Empty(
                            newList = false,
                            searchTerm = emptySearchString
                        )
                )
            }

            // Assert
            onNodeWithText(NO_LOCATIONS_FOUND_TEXT).assertExists()
        }

    @Test
    fun givenSaveIsEnabledWhenSaveClickedShouldCallOnSaveClick() =
        composeExtension.use {
            // Arrange
            val mockOnSaveClick: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Data(
                            newList = false,
                            availableLocations = DUMMY_RELAY_COUNTRIES,
                            saveEnabled = true,
                        ),
                    onSaveClick = mockOnSaveClick
                )
            }

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
            setContentWithTheme {
                CustomListLocationsScreen(
                    uiState =
                        CustomListLocationsUiState.Content.Data(
                            newList = false,
                            availableLocations = DUMMY_RELAY_COUNTRIES,
                            saveEnabled = false,
                        ),
                    onSaveClick = mockOnSaveClick
                )
            }

            // Act
            onNodeWithTag(SAVE_BUTTON_TEST_TAG).performClick()

            // Assert
            verify(exactly = 0) { mockOnSaveClick() }
        }

    companion object {
        const val ADD_LOCATIONS_TEXT = "Add locations"
        const val EDIT_LOCATIONS_TEXT = "Edit locations"
        const val SEARCH_PLACEHOLDER = "Search for..."
        const val EMPTY_SEARCH_FIRST_ROW = "No result for %s."
        const val EMPTY_SEARCH_SECOND_ROW = "Try a different search"
        const val NO_LOCATIONS_FOUND_TEXT = "No locations found"
    }
}
