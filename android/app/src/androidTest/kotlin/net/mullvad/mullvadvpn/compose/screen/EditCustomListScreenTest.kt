package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DUMMY_CUSTOM_LISTS
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class EditCustomListScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { EditCustomListScreen(state = EditCustomListState.Loading) }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun givenNotFoundStateShouldShowNotFound() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { EditCustomListScreen(state = EditCustomListState.NotFound) }

            // Assert
            onNodeWithText(NOT_FOUND_TEXT).assertExists()
        }

    @Test
    fun givenContentStateShouldShowNameFromState() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]
            setContentWithTheme {
                EditCustomListScreen(
                    state =
                        EditCustomListState.Content(
                            id = customList.id,
                            name = customList.name,
                            locations = customList.locations
                        )
                )
            }

            // Assert
            onNodeWithText(customList.name.value)
        }

    @Test
    fun givenContentStateShouldShowNumberOfLocationsFromState() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]
            setContentWithTheme {
                EditCustomListScreen(
                    state =
                        EditCustomListState.Content(
                            id = customList.id,
                            name = customList.name,
                            locations = customList.locations
                        )
                )
            }

            // Assert
            onNodeWithText(LOCATIONS_TEXT.format(customList.locations.size))
        }

    @Test
    fun whenClickingOnDeleteDropdownShouldCallOnDeleteList() =
        composeExtension.use {
            // Arrange
            val mockedOnDelete: (CustomListName) -> Unit = mockk(relaxed = true)
            val customList = DUMMY_CUSTOM_LISTS[0]
            setContentWithTheme {
                EditCustomListScreen(
                    state =
                        EditCustomListState.Content(
                            id = customList.id,
                            name = customList.name,
                            locations = customList.locations
                        ),
                    onDeleteList = mockedOnDelete
                )
            }

            // Act
            onNodeWithTag(TOP_BAR_DROPDOWN_BUTTON_TEST_TAG).performClick()
            onNodeWithTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG).performClick()

            // Assert
            verify { mockedOnDelete(customList.name) }
        }

    @Test
    fun whenClickingOnNameCellShouldCallOnNameClicked() =
        composeExtension.use {
            // Arrange
            val mockedOnNameClicked: (CustomListId, CustomListName) -> Unit = mockk(relaxed = true)
            val customList = DUMMY_CUSTOM_LISTS[0]
            setContentWithTheme {
                EditCustomListScreen(
                    state =
                        EditCustomListState.Content(
                            id = customList.id,
                            name = customList.name,
                            locations = customList.locations
                        ),
                    onNameClicked = mockedOnNameClicked
                )
            }

            // Act
            onNodeWithText(customList.name.value).performClick()

            // Assert
            verify { mockedOnNameClicked(customList.id, customList.name) }
        }

    @Test
    fun whenClickingOnLocationCellShouldCallOnLocationsClicked() =
        composeExtension.use {
            // Arrange
            val mockedOnLocationsClicked: (CustomListId) -> Unit = mockk(relaxed = true)
            val customList = DUMMY_CUSTOM_LISTS[0]
            setContentWithTheme {
                EditCustomListScreen(
                    state =
                        EditCustomListState.Content(
                            id = customList.id,
                            name = customList.name,
                            locations = customList.locations
                        ),
                    onLocationsClicked = mockedOnLocationsClicked
                )
            }

            // Act
            onNodeWithText(LOCATIONS_TEXT.format(customList.locations.size)).performClick()

            // Assert
            verify { mockedOnLocationsClicked(customList.id) }
        }

    companion object {
        const val NOT_FOUND_TEXT = "Not found"
        const val LOCATIONS_TEXT = "%d locations"
    }
}
