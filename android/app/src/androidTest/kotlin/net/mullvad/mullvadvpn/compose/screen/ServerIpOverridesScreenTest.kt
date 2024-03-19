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

    @Suppress("TestFunctionName")
    @Composable
    private fun ScreenWithDefault(
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
    fun ensure_overrides_inactive_is_displayed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ScreenWithDefault(state = ServerIpOverridesViewState.Loaded(false))
            }

            // Assert
            onNodeWithText("Overrides inactive").assertExists()
        }

    @Test
    fun ensure_overrides_active_is_displayed() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ScreenWithDefault(state = ServerIpOverridesViewState.Loaded(true))
            }

            // Assert
            onNodeWithText("Overrides active").assertExists()
        }

    @Test
    fun ensure_overrides_active_shows_warning_on_import() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                ScreenWithDefault(state = ServerIpOverridesViewState.Loaded(true))
            }

            // Act
            onNodeWithTag(testTag = SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()

            // Assert
            onNodeWithText(
                    "Importing new overrides might replace some previously imported overrides."
                )
                .assertExists()
        }

    @Test
    fun ensure_info_click_works() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ScreenWithDefault(
                    state = ServerIpOverridesViewState.Loaded(false),
                    onInfoClick = clickHandler
                )
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_INFO_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensure_reset_click_works() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ScreenWithDefault(
                    state = ServerIpOverridesViewState.Loaded(true),
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
    fun ensure_import_by_file_works() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ScreenWithDefault(
                    state = ServerIpOverridesViewState.Loaded(false),
                    onImportByFile = clickHandler
                )
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensure_import_by_text() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                ScreenWithDefault(
                    state = ServerIpOverridesViewState.Loaded(false),
                    onImportByText = clickHandler
                )
            }

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }
}
