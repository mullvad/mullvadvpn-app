package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.UpdateCustomListUiState
import net.mullvad.mullvadvpn.compose.test.EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.model.CustomListsError
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class EditCustomListNameDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun givenNoErrorShouldShowNoErrorMessage() =
        composeExtension.use {
            // Arrange
            val uiState = UpdateCustomListUiState(error = null)
            setContentWithTheme { EditCustomListNameDialog(uiState = uiState) }

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertDoesNotExist()
            onNodeWithText(OTHER_ERROR_TEXT).assertDoesNotExist()
        }

    @Test
    fun givenCustomListExistsShouldShowCustomListExitsErrorText() =
        composeExtension.use {
            // Arrange
            val uiState = UpdateCustomListUiState(error = CustomListsError.CustomListExists)
            setContentWithTheme { EditCustomListNameDialog(uiState = uiState) }

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertExists()
            onNodeWithText(OTHER_ERROR_TEXT).assertDoesNotExist()
        }

    @Test
    fun givenOtherCustomListErrorShouldShowAnErrorOccurredErrorText() =
        composeExtension.use {
            // Arrange
            val uiState = UpdateCustomListUiState(error = CustomListsError.OtherError)
            setContentWithTheme { EditCustomListNameDialog(uiState = uiState) }

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertDoesNotExist()
            onNodeWithText(OTHER_ERROR_TEXT).assertExists()
        }

    @Test
    fun whenCancelIsClickedShouldDismissDialog() =
        composeExtension.use {
            // Arrange
            val mockedOnDismiss: () -> Unit = mockk(relaxed = true)
            val uiState = UpdateCustomListUiState()
            setContentWithTheme {
                EditCustomListNameDialog(uiState = uiState, onDismiss = mockedOnDismiss)
            }

            // Act
            onNodeWithText(CANCEL_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedOnDismiss.invoke() }
        }

    @Test
    fun givenEmptyTextInputWhenSaveIsClickedThenShouldNotCallUpdateName() =
        composeExtension.use {
            // Arrange
            val mockedUpdateName: (String) -> Unit = mockk(relaxed = true)
            val uiState = UpdateCustomListUiState()
            setContentWithTheme {
                EditCustomListNameDialog(uiState = uiState, updateName = mockedUpdateName)
            }

            // Act
            onNodeWithText(SAVE_BUTTON_TEXT).performClick()

            // Assert
            verify(exactly = 0) { mockedUpdateName.invoke(any()) }
        }

    @Test
    fun givenValidTextInputWhenSaveIsClickedThenShouldCallUpdateName() =
        composeExtension.use {
            // Arrange
            val mockedUpdateName: (String) -> Unit = mockk(relaxed = true)
            val inputText = "NEW NAME"
            val uiState = UpdateCustomListUiState()
            setContentWithTheme {
                EditCustomListNameDialog(uiState = uiState, updateName = mockedUpdateName)
            }

            // Act
            onNodeWithTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG).performTextInput(inputText)
            onNodeWithText(SAVE_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedUpdateName.invoke(inputText) }
        }

    @Test
    fun whenInputIsChangedShouldCallOnInputChanged() =
        composeExtension.use {
            // Arrange
            val mockedOnInputChanged: () -> Unit = mockk(relaxed = true)
            val inputText = "NEW NAME"
            val uiState = UpdateCustomListUiState()
            setContentWithTheme {
                EditCustomListNameDialog(uiState = uiState, onInputChanged = mockedOnInputChanged)
            }

            // Act
            onNodeWithTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG).performTextInput(inputText)

            // Assert
            verify { mockedOnInputChanged.invoke() }
        }

    companion object {
        private const val NAME_EXIST_ERROR_TEXT = "Name is already taken."
        private const val OTHER_ERROR_TEXT = "An error occurred."
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val SAVE_BUTTON_TEXT = "Save"
    }
}
