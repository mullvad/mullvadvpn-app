package net.mullvad.mullvadvpn.compose.dialog

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
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.EditCustomListNameUiState
import net.mullvad.mullvadvpn.compose.test.EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.UnknownCustomListError
import net.mullvad.mullvadvpn.usecase.customlists.RenameError
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

    private fun ComposeContext.initDialog(
        state: EditCustomListNameUiState = EditCustomListNameUiState(),
        updateName: (String) -> Unit = {},
        onInputChanged: (String) -> Unit = {},
        onDismiss: () -> Unit = {},
    ) {
        setContentWithTheme {
            EditCustomListNameDialog(
                state = state,
                updateName = updateName,
                onInputChanged = onInputChanged,
                onDismiss = onDismiss,
            )
        }
    }

    @Test
    fun givenNoErrorShouldShowNoErrorMessage() =
        composeExtension.use {
            // Arrange
            val state = EditCustomListNameUiState(error = null)
            initDialog(state = state)

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertDoesNotExist()
            onNodeWithText(OTHER_ERROR_TEXT).assertDoesNotExist()
        }

    @Test
    fun givenCustomListExistsShouldShowCustomListExitsErrorText() =
        composeExtension.use {
            // Arrange
            val state =
                EditCustomListNameUiState(
                    error = RenameError(NameAlreadyExists(CustomListName.fromString("name")))
                )
            initDialog(state = state)

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertExists()
            onNodeWithText(OTHER_ERROR_TEXT).assertDoesNotExist()
        }

    @Test
    fun givenOtherCustomListErrorShouldShowAnErrorOccurredErrorText() =
        composeExtension.use {
            // Arrange
            val state =
                EditCustomListNameUiState(
                    error = RenameError(UnknownCustomListError(RuntimeException("")))
                )
            initDialog(state = state)

            // Assert
            onNodeWithText(NAME_EXIST_ERROR_TEXT).assertDoesNotExist()
            onNodeWithText(OTHER_ERROR_TEXT).assertExists()
        }

    @Test
    fun whenCancelIsClickedShouldDismissDialog() =
        composeExtension.use {
            // Arrange
            val mockedOnDismiss: () -> Unit = mockk(relaxed = true)
            val state = EditCustomListNameUiState()
            initDialog(state = state, onDismiss = mockedOnDismiss)

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
            val state = EditCustomListNameUiState()
            initDialog(state = state, updateName = mockedUpdateName)

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
            val state = EditCustomListNameUiState(name = inputText)
            initDialog(state, updateName = mockedUpdateName)

            // Act
            onNodeWithText(SAVE_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedUpdateName.invoke(inputText) }
        }

    @Test
    fun whenInputIsChangedShouldCallOnInputChanged() =
        composeExtension.use {
            // Arrange
            val mockedOnInputChanged: (String) -> Unit = mockk(relaxed = true)
            val inputText = "NEW NAME"
            val state = EditCustomListNameUiState()
            initDialog(state, onInputChanged = mockedOnInputChanged)

            // Act
            onNodeWithTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG).performTextInput(inputText)

            // Assert
            verify { mockedOnInputChanged.invoke(inputText) }
        }

    companion object {
        private const val NAME_EXIST_ERROR_TEXT = "Name is already taken."
        private const val OTHER_ERROR_TEXT = "An error occurred."
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val SAVE_BUTTON_TEXT = "Save"
    }
}
