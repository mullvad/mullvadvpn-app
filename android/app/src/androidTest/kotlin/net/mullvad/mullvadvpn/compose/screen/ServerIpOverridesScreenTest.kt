package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
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
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesUiState
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

    private fun ComposeContext.initScreen(
        state: ServerIpOverridesUiState,
        onBackClick: () -> Unit = {},
        onInfoClick: () -> Unit = {},
        onResetOverridesClick: () -> Unit = {},
        onImportByFile: () -> Unit = {},
        onImportByText: () -> Unit = {},
    ) {
        setContentWithTheme {
            ServerIpOverridesScreen(
                state = state,
                onBackClick = onBackClick,
                onInfoClick = onInfoClick,
                onResetOverridesClick = onResetOverridesClick,
                onImportByFile = onImportByFile,
                onImportByText = onImportByText,
            )
        }
    }

    @Test
    fun ensureOverridesInactiveIsDisplayed() =
        composeExtension.use {
            // Arrange
            initScreen(state = ServerIpOverridesUiState.Loaded(false))

            // Assert
            onNodeWithText("Overrides inactive").assertExists()
        }

    @Test
    fun ensureOverridesActiveIsDisplayed() =
        composeExtension.use {
            // Arrange
            initScreen(state = ServerIpOverridesUiState.Loaded(true))

            // Assert
            onNodeWithText("Overrides active").assertExists()
        }

    @Test
    fun ensureOverridesActiveShowsWarningOnImport() =
        composeExtension.use {
            // Arrange
            initScreen(state = ServerIpOverridesUiState.Loaded(true))

            // Act
            onNodeWithTag(testTag = SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()

            // Assert
            onNodeWithText(
                    "Importing new overrides might replace some previously imported overrides."
                )
                .assertExists()
        }

    @Test
    fun ensureInfoClickWorks() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(state = ServerIpOverridesUiState.Loaded(false), onInfoClick = clickHandler)

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_INFO_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensureResetClickWorks() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = ServerIpOverridesUiState.Loaded(true),
                onResetOverridesClick = clickHandler,
            )

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_MORE_VERT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDE_RESET_OVERRIDES_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensureImportByFileWorks() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = ServerIpOverridesUiState.Loaded(false),
                onImportByFile = clickHandler,
            )

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }

    @Test
    fun ensureImportByText() =
        composeExtension.use {
            // Arrange
            val clickHandler: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = ServerIpOverridesUiState.Loaded(false),
                onImportByText = clickHandler,
            )

            // Act
            onNodeWithTag(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG).performClick()
            onNodeWithTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG).performClick()

            // Assert
            verify { clickHandler() }
        }
}
