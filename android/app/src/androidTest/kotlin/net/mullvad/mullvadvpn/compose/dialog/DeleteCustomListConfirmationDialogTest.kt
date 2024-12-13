package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.DeleteCustomListUiState
import net.mullvad.mullvadvpn.lib.model.CustomListName
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class DeleteCustomListConfirmationDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initDialog(
        state: DeleteCustomListUiState =
            DeleteCustomListUiState(CustomListName.fromString("My Custom List"), null),
        onDelete: () -> Unit = {},
        onBack: () -> Unit = {},
    ) {
        setContentWithTheme {
            DeleteCustomListConfirmationDialog(state = state, onDelete = onDelete, onBack = onBack)
        }
    }

    @Test
    fun givenNameShouldShowDeleteNameTitle() =
        composeExtension.use {
            // Arrange
            val name = CustomListName.fromString("List should be deleted")
            initDialog(state = DeleteCustomListUiState(name = name, deleteError = null))

            // Assert
            onNodeWithText(DELETE_TITLE.format(name)).assertExists()
        }

    @Test
    fun whenDeleteIsClickedShouldCallOnDelete() =
        composeExtension.use {
            // Arrange
            val name = CustomListName.fromString("List should be deleted")
            val mockedOnDelete: () -> Unit = mockk(relaxed = true)
            initDialog(
                state = DeleteCustomListUiState(name = name, deleteError = null),
                onDelete = mockedOnDelete,
            )

            // Act
            onNodeWithText(DELETE_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedOnDelete.invoke() }
        }

    @Test
    fun whenCancelIsClickedShouldCallOnBack() =
        composeExtension.use {
            // Arrange
            val name = CustomListName.fromString("List should be deleted")
            val mockedOnBack: () -> Unit = mockk(relaxed = true)
            initDialog(
                state = DeleteCustomListUiState(name = name, deleteError = null),
                onBack = mockedOnBack,
            )

            // Act
            onNodeWithText(CANCEL_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedOnBack.invoke() }
        }

    companion object {
        private const val DELETE_TITLE = "Delete \"%s\"?"
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val DELETE_BUTTON_TEXT = "Delete"
    }
}
