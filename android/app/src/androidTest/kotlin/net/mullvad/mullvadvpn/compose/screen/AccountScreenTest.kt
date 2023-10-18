package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class AccountScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Test
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("Redeem voucher").assertExists()
            onNodeWithText("Log out").assertExists()
        }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Test
    fun testManageAccountClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onManageAccountClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Manage account").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Test
    fun testRedeemVoucherClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onRedeemVoucherClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Redeem voucher").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Test
    fun testLogoutClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onLogoutClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Log out").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    companion object {
        private const val DUMMY_DEVICE_NAME = "fake_name"
        private const val DUMMY_ACCOUNT_NUMBER = "fake_number"
    }
}
