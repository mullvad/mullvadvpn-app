package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.data.DUMMY_CUSTOM_LISTS
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.NEW_LIST_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.CustomList
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

    private fun ComposeContext.initScreen(
        state: CustomListsUiState = CustomListsUiState.Loading,
        addCustomList: () -> Unit = {},
        openCustomList: (CustomList) -> Unit = {},
        onBackClick: () -> Unit = {},
    ) {

        setContentWithTheme {
            CustomListsScreen(
                state = state,
                addCustomList = addCustomList,
                openCustomList = openCustomList,
                onBackClick = onBackClick,
            )
        }
    }

    @Test
    fun givenLoadingStateShouldShowLoadingSpinner() =
        composeExtension.use {
            // Arrange
            initScreen(state = CustomListsUiState.Loading)

            // Assert
            onNodeWithTag(CIRCULAR_PROGRESS_INDICATOR).assertExists()
        }

    @Test
    fun givenCustomListsShouldShowTheirNames() =
        composeExtension.use {
            // Arrange
            val customLists = DUMMY_CUSTOM_LISTS
            initScreen(state = CustomListsUiState.Content(customLists = customLists))

            // Assert
            onNodeWithText(customLists[0].name.value).assertExists()
            onNodeWithText(customLists[1].name.value).assertExists()
        }

    @Test
    fun whenNewListButtonIsClickedShouldCallAddCustomList() =
        composeExtension.use {
            // Arrange
            val customLists = DUMMY_CUSTOM_LISTS
            val mockedAddCustomList: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = CustomListsUiState.Content(customLists = customLists),
                addCustomList = mockedAddCustomList,
            )

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
            val mockedOpenCustomList: (CustomList) -> Unit = mockk(relaxed = true)
            initScreen(
                state = CustomListsUiState.Content(customLists = customLists),
                openCustomList = mockedOpenCustomList,
            )

            // Act
            onNodeWithText(clickedList.name.value).performClick()

            // Assert
            verify { mockedOpenCustomList(clickedList) }
        }
}
