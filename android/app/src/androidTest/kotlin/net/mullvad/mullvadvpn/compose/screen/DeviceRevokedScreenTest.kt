package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.compose.component.AppTheme
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class DeviceRevokedScreenTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @MockK
    lateinit var mockedViewModel: DeviceRevokedViewModel

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        every { mockedViewModel.onGoToLoginClicked() } just Runs
    }

    @Test
    fun testUnblockWarningShowingWhenSecured() {
        // Arrange
        every {
            mockedViewModel.uiState
        } returns MutableStateFlow(DeviceRevokedUiState(isSecured = true))

        // Act
        composeTestRule.setContent {
            AppTheme {
                DeviceRevokedScreen(mockedViewModel)
            }
        }

        // Assert
        composeTestRule
            .onNodeWithText(UNBLOCK_WARNING)
            .assertExists()
    }

    @Test
    fun testUnblockWarningNotShowingWhenNotSecured() {
        // Arrange
        every {
            mockedViewModel.uiState
        } returns MutableStateFlow(DeviceRevokedUiState(isSecured = false))

        // Act
        composeTestRule.setContent {
            AppTheme {
                DeviceRevokedScreen(mockedViewModel)
            }
        }

        // Assert
        composeTestRule
            .onNodeWithText(UNBLOCK_WARNING)
            .assertDoesNotExist()
    }

    @Test
    fun testGoToLogin() {
        // Arrange
        every {
            mockedViewModel.uiState
        } returns MutableStateFlow(DeviceRevokedUiState(isSecured = false))
        composeTestRule.setContent {
            AppTheme {
                DeviceRevokedScreen(mockedViewModel)
            }
        }

        // Act
        composeTestRule
            .onNodeWithText(GO_TO_LOGIN_BUTTON_TEXT)
            .performClick()

        // Assert
        verify { mockedViewModel.onGoToLoginClicked() }
    }

    companion object {
        private const val GO_TO_LOGIN_BUTTON_TEXT = "Go to login"
        private const val UNBLOCK_WARNING =
            "Going to login will unblock the internet on this device."
    }
}
