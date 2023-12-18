package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SettingsScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun testLoggedInState() {
        // Arrange
        composeTestRule.setContentWithTheme {
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
        composeTestRule.apply {
            onNodeWithText("VPN settings").assertExists()
            onNodeWithText("Split tunneling").assertExists()
            onNodeWithText("App version").assertExists()
        }
    }

    @Test
    @OptIn(ExperimentalMaterial3Api::class)
    fun testLoggedOutState() {
        // Arrange
        composeTestRule.setContentWithTheme {
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
        composeTestRule.apply {
            onNodeWithText("VPN settings").assertDoesNotExist()
            onNodeWithText("Split tunneling").assertDoesNotExist()
            onNodeWithText("App version").assertExists()
        }
    }
}
