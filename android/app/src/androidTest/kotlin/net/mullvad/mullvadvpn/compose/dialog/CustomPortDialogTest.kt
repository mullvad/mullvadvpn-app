package net.mullvad.mullvadvpn.compose.dialog

import android.annotation.SuppressLint
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.test.CUSTOM_PORT_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.model.PortRange
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
    fun testShowWireguardCustomPortDialogInvalidInt() =
        composeExtension.use {
            // Input a number to make sure that a too long number does not show and it does not
            // crash
            // the app

            // Arrange
            setContentWithTheme { testWireguardCustomPortDialog() }

            // Act
            onNodeWithTag(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG).performTextInput(invalidCustomPort)

            // Assert
            onNodeWithTagAndText(CUSTOM_PORT_DIALOG_INPUT_TEST_TAG, invalidCustomPort)
                .assertDoesNotExist()
        }

    companion object {
        const val invalidCustomPort = "21474836471"
    }
}
