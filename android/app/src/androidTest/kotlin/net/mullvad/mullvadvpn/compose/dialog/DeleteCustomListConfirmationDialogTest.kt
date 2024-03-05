package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
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

    @Test
    fun givenNameShouldShowDeleteNameTitle() =
        composeExtension.use {
            // Arrange
            val name = "List should be deleted"
            setContentWithTheme { DeleteCustomListConfirmationDialog(name = name) }

            // Assert
            onNodeWithText(DELETE_TITLE.format(name)).assertExists()
        }

    @Test
    fun whenDeleteIsClickedShouldCallOnDelete() =
        composeExtension.use {
            // Arrange
            val name = "List should be deleted"
            val mockedOnDelete: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                DeleteCustomListConfirmationDialog(name = name, onDelete = mockedOnDelete)
            }

            // Act
            onNodeWithText(DELETE_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedOnDelete.invoke() }
        }

    @Test
    fun whenCancelIsClickedShouldCallOnBack() =
        composeExtension.use {
            // Arrange
            val name = "List should be deleted"
            val mockedOnBack: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                DeleteCustomListConfirmationDialog(name = name, onBack = mockedOnBack)
            }

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
