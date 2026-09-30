package net.mullvad.mullvadvpn.feature.account.impl

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@ExperimentalTestApi
@OptIn(ExperimentalMaterial3Api::class)
class AccountScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    private fun ComposeContext.initScreen(
        state: AccountUiState? = null,
        onCopyAccountNumber: (String) -> Unit = {},
        onLogoutClick: () -> Unit = {},
        onPlayPaymentInfoClick: (PaymentStatus) -> Unit = {},
        onBackClick: () -> Unit = {},
        onManageDevicesClick: () -> Unit = {},
        navigateToDeleteAccount: () -> Unit = {},
        navigateToAddTime: () -> Unit = {},
    ) {
        setContentWithTheme {
            AccountScreen(
                state = state,
                onCopyAccountNumber = onCopyAccountNumber,
                onManageDevicesClick = onManageDevicesClick,
                onLogoutClick = onLogoutClick,
                onPlayPaymentInfoClick = onPlayPaymentInfoClick,
                onBackClick = onBackClick,
                navigateToDeleteAccount = navigateToDeleteAccount,
                navigateToAddTime = navigateToAddTime,
            )
        }
    }

    @Test
    fun testDefaultState() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                AccountUiState(
                    deviceName = DUMMY_DEVICE_NAME,
                    accountNumber = DUMMY_ACCOUNT_NUMBER,
                    accountExpiry = null,
                    showLogoutLoading = false,
                    paymentStatus = null,
                )
        )

        // Assert
        onNodeWithText("Log out").assertExists()
    }

    @Test
    fun testLogoutClick() = composeExtension.use {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        initScreen(
            state =
                AccountUiState(
                    deviceName = DUMMY_DEVICE_NAME,
                    accountNumber = DUMMY_ACCOUNT_NUMBER,
                    accountExpiry = null,
                    showLogoutLoading = false,
                    paymentStatus = null,
                ),
            onLogoutClick = mockedClickHandler,
        )

        // Act
        onNodeWithText("Log out").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testShowVerificationInProgress() = composeExtension.use {
        // Arrange
        initScreen(
            state =
                AccountUiState(
                    deviceName = DUMMY_DEVICE_NAME,
                    accountNumber = DUMMY_ACCOUNT_NUMBER,
                    accountExpiry = null,
                    showLogoutLoading = false,
                    paymentStatus = PaymentStatus.PENDING,
                )
        )

        // Assert
        onNodeWithText("Google Play payment pending").assertExists()
    }

    companion object {
        private const val DUMMY_DEVICE_NAME = "fake_name"
        private val DUMMY_ACCOUNT_NUMBER = AccountNumber("1234123412341234")
    }
}
