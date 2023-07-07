package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class DeviceRevokedScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testUnblockWarningShowingWhenSecured() {
        // Arrange
        val state = DeviceRevokedUiState.SECURED

        // Act
        composeTestRule.setContent { AppTheme { DeviceRevokedScreen(state) } }

        // Assert
        composeTestRule.onNodeWithText(UNBLOCK_WARNING).assertExists()
    }

    @Test
    fun testUnblockWarningNotShowingWhenNotSecured() {
        // Arrange
        val state = DeviceRevokedUiState.UNSECURED

        // Act
        composeTestRule.setContent { AppTheme { DeviceRevokedScreen(state) } }

        // Assert
        composeTestRule.onNodeWithText(UNBLOCK_WARNING).assertDoesNotExist()
    }

    @Test
    fun testGoToLogin() {
        // Arrange
        val state = DeviceRevokedUiState.UNSECURED
        val mockOnGoToLoginClicked: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AppTheme {
                DeviceRevokedScreen(state = state, onGoToLoginClicked = mockOnGoToLoginClicked)
            }
        }

        // Act
        composeTestRule.onNodeWithText(GO_TO_LOGIN_BUTTON_TEXT).performClick()

        // Assert
        verify { mockOnGoToLoginClicked.invoke() }
    }

    companion object {
        private const val GO_TO_LOGIN_BUTTON_TEXT = "Go to login"
        private const val UNBLOCK_WARNING =
            "Going to login will unblock the internet on this device."
    }
}
