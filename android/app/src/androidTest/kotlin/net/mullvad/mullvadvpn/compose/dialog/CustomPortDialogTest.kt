package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.onNodeWithTagAndText
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class CustomPortDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @SuppressLint("ComposableNaming")
    @Composable
    private fun testWireguardCustomPortDialog(
        initialPort: Int? = null,
        allowedPortRanges: List<PortRange> = emptyList(),
        onSave: (Int?) -> Unit = { _ -> },
        onDismiss: () -> Unit = {},
    ) {

        WireguardCustomPortDialog(
            initialPort = initialPort,
            allowedPortRanges = allowedPortRanges,
            onSave = onSave,
            onDismiss = onDismiss
        )
    }

    @Test
    fun testShowWireguardCustomPortDialogInvalidInt() {
        // Input a number to make sure that a too long number does not show and it does not crash
        // the app

        // Arrange
        composeTestRule.setContentWithTheme { testWireguardCustomPortDialog() }

        // Act
        composeTestRule
            .onNodeWithTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG)
            .performTextInput(invalidCustomPort)

        // Assert
        composeTestRule
            .onNodeWithTagAndText(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG, invalidCustomPort)
            .assertDoesNotExist()
    }

    companion object {
        const val invalidCustomPort = "21474836471"
    }
}
