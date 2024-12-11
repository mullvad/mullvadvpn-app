package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.MtuDialogUiState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class MtuDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private val defaultState =
        MtuDialogUiState(mtuInput = "", isValidInput = true, showResetToDefault = true)

    private fun ComposeContext.initDialog(
        state: MtuDialogUiState = defaultState,
        onInputChanged: (String) -> Unit = {},
        onSaveMtu: (String) -> Unit = {},
        onResetMtu: () -> Unit = {},
        onDismiss: () -> Unit = {},
    ) {
        setContentWithTheme {
            MtuDialog(
                state = state,
                onInputChanged = onInputChanged,
                onSaveMtu = onSaveMtu,
                onResetMtu = onResetMtu,
                onDismiss = onDismiss,
            )
        }
    }

    @Test
    fun testMtuDialogWithDefaultValue() =
        composeExtension.use {
            // Arrange
            initDialog()

            // Assert
            onNodeWithText(EMPTY_STRING).assertExists()
        }

    @Test
    fun testMtuDialogWithEditValue() =
        composeExtension.use {
            // Arrange
            initDialog(defaultState.copy(mtuInput = VALID_DUMMY_MTU_VALUE))

            // Assert
            onNodeWithText(VALID_DUMMY_MTU_VALUE).assertExists()
        }

    @Test
    fun testMtuDialogSubmitOfValidValue() =
        composeExtension.use {
            // Arrange
            val mockedSubmitHandler: (String) -> Unit = mockk(relaxed = true)
            initDialog(
                defaultState.copy(mtuInput = VALID_DUMMY_MTU_VALUE),
                onSaveMtu = mockedSubmitHandler,
            )

            // Act
            onNodeWithText("Submit").assertIsEnabled().performClick()

            // Assert
            verify { mockedSubmitHandler.invoke(VALID_DUMMY_MTU_VALUE) }
        }

    @Test
    fun testMtuDialogSubmitButtonDisabledWhenInvalidInput() =
        composeExtension.use {
            // Arrange
            initDialog(defaultState.copy(mtuInput = INVALID_DUMMY_MTU_VALUE, isValidInput = false))

            // Assert
            onNodeWithText("Submit").assertIsNotEnabled()
        }

    @Test
    fun testMtuDialogResetClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initDialog(
                defaultState.copy(mtuInput = VALID_DUMMY_MTU_VALUE),
                onResetMtu = mockedClickHandler,
            )

            // Act
            onNodeWithText("Reset to default").performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }

    @Test
    fun testMtuDialogCancelClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initDialog(onDismiss = mockedClickHandler)

            // Assert
            onNodeWithText("Cancel").performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }

    companion object {
        private const val EMPTY_STRING = ""
        private const val VALID_DUMMY_MTU_VALUE = "1337"
        private const val INVALID_DUMMY_MTU_VALUE = "1111"
    }
}
