package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import de.mannodermaus.junit5.compose.createComposeExtension
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class SettingsScreenTest {
    @JvmField @RegisterExtension val composeExtension = createComposeExtension()

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
                    uiState =
                        SettingsUiState(
                            appVersion = "",
                            isLoggedIn = true,
                            isUpdateAvailable = true,
                            isPlayBuild = false
                        ),
                )
            }
            // Assert
            onNodeWithText("VPN settings").assertExists()
            onNodeWithText("Split tunneling").assertExists()
            onNodeWithText("App version").assertExists()
        }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun testLoggedOutState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                SettingsScreen(
                    uiState =
                        SettingsUiState(
                            appVersion = "",
                            isLoggedIn = false,
                            isUpdateAvailable = true,
                            isPlayBuild = false
                        ),
                )
            }
            // Assert
            onNodeWithText("VPN settings").assertDoesNotExist()
            onNodeWithText("Split tunneling").assertDoesNotExist()
            onNodeWithText("App version").assertExists()
        }
}
