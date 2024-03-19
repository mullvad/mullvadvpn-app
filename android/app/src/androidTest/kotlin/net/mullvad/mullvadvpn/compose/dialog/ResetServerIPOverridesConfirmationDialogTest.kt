package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.test.RESET_SERVER_IP_OVERRIDE_CANCEL_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.RESET_SERVER_IP_OVERRIDE_RESET_TEST_TAG
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ResetServerIPOverridesConfirmationDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun ensure_cancel_click_works() =
        composeExtension.use {
            val clickHandler: () -> Unit = mockk(relaxed = true)

            // Arrange
            setContentWithTheme {
                ResetServerIpOverridesConfirmationDialog(
                    onNavigateBack = clickHandler,
                    onClearAllOverrides = {}
                )
            }

            // Act
            onNodeWithTag(RESET_SERVER_IP_OVERRIDE_CANCEL_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensure_reset_click_works() =
        composeExtension.use {
            val clickHandler: () -> Unit = mockk(relaxed = true)

            // Arrange
            setContentWithTheme {
                ResetServerIpOverridesConfirmationDialog(
                    onNavigateBack = {},
                    onClearAllOverrides = clickHandler
                )
            }

            // Act
            onNodeWithTag(RESET_SERVER_IP_OVERRIDE_RESET_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }
}
