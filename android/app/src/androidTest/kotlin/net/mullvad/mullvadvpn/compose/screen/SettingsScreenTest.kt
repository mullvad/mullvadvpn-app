package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.lib.theme.DarkThemeState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class SettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun testLoggedInState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SettingsScreen(
                    state =
                        SettingsUiState(
                            appVersion = "",
                            isLoggedIn = true,
                            isSupportedVersion = true,
                            isPlayBuild = false,
                            isMaterialYouTheme = false,
                            darkThemeState = DarkThemeState.OFF
                        ),
                )
            }
            // Assert
            onNodeWithText("VPN settings").assertExists()
            onNodeWithText("Split tunneling").assertExists()
            onNodeWithText("App version").assertExists()
            onNodeWithText("API access").assertExists()
        }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun testLoggedOutState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SettingsScreen(
                    state =
                        SettingsUiState(
                            appVersion = "",
                            isLoggedIn = false,
                            isSupportedVersion = true,
                            isPlayBuild = false,
                            isMaterialYouTheme = false,
                            darkThemeState = DarkThemeState.OFF
                        ),
                )
            }
            // Assert
            onNodeWithText("VPN settings").assertDoesNotExist()
            onNodeWithText("Split tunneling").assertDoesNotExist()
            onNodeWithText("App version").assertExists()
            onNodeWithText("API access").assertExists()
        }
}
