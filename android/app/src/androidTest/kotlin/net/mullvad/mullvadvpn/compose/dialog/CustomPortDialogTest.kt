package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
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
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class CustomPortDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initDialog(
        title: String = "",
        portInput: String = "",
        isValidInput: Boolean = false,
        allowedPortRanges: List<PortRange> = emptyList(),
        recommendedPortRanges: List<PortRange> = emptyList(),
        showResetToDefault: Boolean = false,
        onInputChanged: (String) -> Unit = { _ -> },
        onSavePort: (String) -> Unit = { _ -> },
        onResetPort: () -> Unit = {},
        onDismiss: () -> Unit = {},
    ) {
        setContentWithTheme {
            CustomPortDialog(
                title = title,
                portInput = portInput,
                isValidInput = isValidInput,
                allowedPortRanges = allowedPortRanges,
                recommendedPortRanges = recommendedPortRanges,
                showResetToDefault = showResetToDefault,
                onInputChanged = onInputChanged,
                onSavePort = onSavePort,
                onDismiss = onDismiss,
                onResetPort = onResetPort,
            )
        }
    }

    @Test
    fun testShowWireguardCustomPortDialogInvalidInt() =
        composeExtension.use {
            // Input a number to make sure that a too long number does not show and it does not
            // crash the app

            // Arrange
            var input by mutableStateOf("")
            initDialog(
                title = "",
                portInput = input,
                isValidInput = false,
                showResetToDefault = false,
                onInputChanged = { input = it },
            )

            // Act
            onNodeWithTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).performTextInput(INVALID_PORT_INPUT)

            // Assert
            onNodeWithTagAndText(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG, INVALID_PORT_INPUT)
                .assertDoesNotExist()
        }

    @Test
    fun testEmptyInputResultsInSetPortButtonBeingDisabled() =
        composeExtension.use {
            // Arrange
            initDialog(isValidInput = false)

            // Assert
            onNodeWithText("Set port").assertIsNotEnabled()
        }

    @Test
    fun testValidInputResultsInSetPortButtonBeingEnabled() =
        composeExtension.use {
            // Arrange
            initDialog(portInput = VALID_CUSTOM_PORT, isValidInput = true)

            // Assert
            onNodeWithText("Set port").assertIsEnabled()
            onNodeWithText(VALID_CUSTOM_PORT).assertExists()
        }

    @Test
    fun testInvalidInputResultsInSetPortButtonBeingDisabled() =
        composeExtension.use {
            // Arrange
            initDialog(portInput = INVALID_CUSTOM_PORT, isValidInput = false)

            // Assert
            onNodeWithText("Set port").assertIsNotEnabled()
        }

    @Test
    fun testDialogSubmitOfValidValue() =
        composeExtension.use {
            // Arrange
            val mockedSubmitHandler: (String) -> Unit = mockk(relaxed = true)
            initDialog(
                portInput = VALID_CUSTOM_PORT,
                isValidInput = true,
                onSavePort = mockedSubmitHandler,
            )

            // Act
            onNodeWithText("Set port").assertIsEnabled().performClick()

            // Assert
            verify { mockedSubmitHandler.invoke(VALID_CUSTOM_PORT) }
        }

    @Test
    fun testDialogResetClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initDialog(
                portInput = VALID_CUSTOM_PORT,
                isValidInput = true,
                showResetToDefault = true,
                onResetPort = mockedClickHandler,
            )

            // Act
            onNodeWithText("Remove custom port").performClick()

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
        const val INVALID_PORT_INPUT = "21474836471"
        const val INVALID_CUSTOM_PORT = "10"
        const val VALID_CUSTOM_PORT = "4001"
    }
}
