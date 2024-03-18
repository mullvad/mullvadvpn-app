package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_IMPORT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@ExperimentalTestApi
class ServerIpOverridesScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Composable
    private fun Screen(
        state: ServerIpOverridesViewState,
        onBackClick: () -> Unit = {},
        onInfoClick: () -> Unit = {},
        onResetOverridesClick: () -> Unit = {},
        onImportByFile: () -> Unit = {},
        onImportByText: () -> Unit = {},
    ) {
        ServerIpOverridesScreen(
            state = state,
            onBackClick = onBackClick,
            onInfoClick = onInfoClick,
            onResetOverridesClick = onResetOverridesClick,
            onImportByFile = onImportByFile,
            onImportByText = onImportByText
        )
    }

    @Test
    fun testOverridesInactive() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { Screen(state = ServerIpOverridesViewState(false)) }

            // Assert
            onNodeWithText("Overrides inactive").assertExists()
        }

    @Test
    fun testOverridesActive() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { Screen(state = ServerIpOverridesViewState(true)) }

            // Assert
            onNodeWithText("Overrides active").assertExists()
        }

    @Test
    fun testOverridesActiveShowsWarningOnImport() =
        composeExtension.use {
            // Arrange
            setContentWithTheme { Screen(state = ServerIpOverridesViewState(true)) }

            // Act
            onNodeWithTag(testTag = SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()

            // Assert
            onNodeWithText(
                    "Importing new overrides might replace some previously imported overrides."
                )
                .assertExists()
        }

    @Test
    fun testInfoClick() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                Screen(state = ServerIpOverridesViewState(false), onInfoClick = clickHandler)
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_INFO_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun testResetClick() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                Screen(
                    state = ServerIpOverridesViewState(false),
                    onResetOverridesClick = clickHandler
                )
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun testImportByFile() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                Screen(state = ServerIpOverridesViewState(false), onImportByFile = clickHandler)
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun testImportByText() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                Screen(state = ServerIpOverridesViewState(false), onImportByText = clickHandler)
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }
}
