package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.SnackbarHostState
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
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.NEW_LIST_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.relaylist.RelayItem
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class CustomListsScreenTest {
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
                CustomListsScreen(
                    state = CustomListsUiState.Loading,
                    snackbarHostState = SnackbarHostState()
                )
            }

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun givenCustomListsShouldShowTheirNames() =
        composeExtension.use {
            // Arrange
            val customLists = DUMMY_CUSTOM_LISTS
            setContentWithTheme {
                CustomListsScreen(
                    state = CustomListsUiState.Content(customLists = customLists),
                    snackbarHostState = SnackbarHostState()
                )
            }

            // Assert
            onNodeWithText(customLists[0].name).assertExists()
            onNodeWithText(customLists[1].name).assertExists()
        }

    @Test
    fun whenNewListButtonIsClickedShouldCallAddCustomList() =
        composeExtension.use {
            // Arrange
            val customLists = DUMMY_CUSTOM_LISTS
            val mockedAddCustomList: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                CustomListsScreen(
                    state = CustomListsUiState.Content(customLists = customLists),
                    snackbarHostState = SnackbarHostState(),
                    addCustomList = mockedAddCustomList
                )
            }

            // Act
            onNodeWithTag(NEW_LIST_BUTTON_TEST_TAG).performClick()

            // Assert
            verify { mockedAddCustomList() }
        }

    @Test
    fun whenACustomListIsClickedShouldCallOpenCustomList() =
        composeExtension.use {
            // Arrange
            val customLists = DUMMY_CUSTOM_LISTS
            val clickedList = DUMMY_CUSTOM_LISTS[0]
            val mockedOpenCustomList: (RelayItem.CustomList) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                CustomListsScreen(
                    state = CustomListsUiState.Content(customLists = customLists),
                    snackbarHostState = SnackbarHostState(),
                    openCustomList = mockedOpenCustomList
                )
            }

            // Act
            onNodeWithText(clickedList.name).performClick()

            // Assert
            verify { mockedOpenCustomList(clickedList) }
        }
}
