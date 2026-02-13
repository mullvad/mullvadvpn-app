package net.mullvad.mullvadvpn.customlist.impl

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import kotlin.text.format
import net.mullvad.mullvadvpn.customlist.impl.data.DUMMY_CUSTOM_LISTS
import net.mullvad.mullvadvpn.customlist.impl.screen.editlist.EditCustomListScreen
import net.mullvad.mullvadvpn.customlist.impl.screen.editlist.EditCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.ui.tag.CIRCULAR_PROGRESS_INDICATOR_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
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

    private fun ComposeContext.initScreen(
        state: EditCustomListUiState = EditCustomListUiState.Loading,
        onDeleteList: (id: CustomListId, name: CustomListName) -> Unit = { _, _ -> },
        onNameClicked: (id: CustomListId, name: CustomListName) -> Unit = { _, _ -> },
        onLocationsClicked: (CustomListId) -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {
        setContentWithTheme {
            EditCustomListScreen(
                state = state,
                onDeleteList = onDeleteList,
                onNameClicked = onNameClicked,
                onLocationsClicked = onLocationsClicked,
                onBackClick = onBackClick,
            )
        }
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            initScreen()

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG).assertExists()
        }

    @Test
    fun givenNotFoundStateShouldShowNotFound() =
        composeExtension.use {
            // Arrange
            initScreen(state = EditCustomListUiState.NotFound)

            // Assert
            onNodeWithText(NOT_FOUND_TEXT).assertExists()
        }

    @Test
    fun givenContentStateShouldShowNameFromState() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]

            initScreen(
                state =
                    EditCustomListUiState.Content(
                        id = customList.id,
                        name = customList.name,
                        locations = customList.locations,
                    )
            )

            // Assert
            onNodeWithText(customList.name.value).assertExists()
        }

    @Test
    fun givenContentStateShouldShowNumberOfLocationsFromState() =
        composeExtension.use {
            // Arrange
            val customList = DUMMY_CUSTOM_LISTS[0]
            initScreen(
                state =
                    EditCustomListUiState.Content(
                        id = customList.id,
                        name = customList.name,
                        locations = customList.locations,
                    )
            )

            // Assert
            onNodeWithText(LOCATIONS_TEXT.format(customList.locations.size)).assertExists()
        }

    @Test
    fun whenClickingOnDeleteDropdownShouldCallOnDeleteList() =
        composeExtension.use {
            // Arrange
            val mockedOnDelete: (CustomListId, CustomListName) -> Unit = mockk(relaxed = true)
            val customList = DUMMY_CUSTOM_LISTS[0]
            initScreen(
                state =
                    EditCustomListUiState.Content(
                        id = customList.id,
                        name = customList.name,
                        locations = customList.locations,
                    ),
                onDeleteList = mockedOnDelete,
            )

            // Act
            onNodeWithTag(TOP_BAR_DROPDOWN_BUTTON_TEST_TAG).performClick()
            onNodeWithTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG).performClick()

            // Assert
            verify { mockedOnDelete(customList.id, customList.name) }
        }

    @Test
    fun whenClickingOnNameCellShouldCallOnNameClicked() =
        composeExtension.use {
            // Arrange
            val mockedOnNameClicked: (CustomListId, CustomListName) -> Unit = mockk(relaxed = true)
            val customList = DUMMY_CUSTOM_LISTS[0]
            initScreen(
                state =
                    EditCustomListUiState.Content(
                        id = customList.id,
                        name = customList.name,
                        locations = customList.locations,
                    ),
                onNameClicked = mockedOnNameClicked,
            )

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
            initScreen(
                state =
                    EditCustomListUiState.Content(
                        id = customList.id,
                        name = customList.name,
                        locations = customList.locations,
                    ),
                onLocationsClicked = mockedOnLocationsClicked,
            )

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
