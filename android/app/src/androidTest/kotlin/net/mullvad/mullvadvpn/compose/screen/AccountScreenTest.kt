package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.state.AccountDialogState
import net.mullvad.mullvadvpn.compose.state.AccountUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

@OptIn(ExperimentalMaterial3Api::class)
class AccountScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("Redeem voucher").assertExists()
            onNodeWithText("Log out").assertExists()
        }
    }

    @Test
    fun testManageAccountClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onManageAccountClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Manage account").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testRedeemVoucherClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onRedeemVoucherClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Redeem voucher").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testLogoutClick() {
        // Arrange
        val mockedClickHandler: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null
                    ),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow(),
                onLogoutClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Log out").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testShowPurchaseCompleteDialog() {
        // Arrange
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState = AccountUiState(dialogState = AccountDialogState.PurchaseComplete),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Time was successfully added").assertExists()
    }

    @Test
    fun testShowVerificationErrorDialog() {
        // Arrange
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState = AccountUiState(dialogState = AccountDialogState.VerificationError),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Payment was unsuccessful").assertExists()
    }

    @Test
    fun testShowBillingErrorDialog() {
        // Arrange
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState = AccountUiState(dialogState = AccountDialogState.BillingError),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Google Play services not available").assertExists()
    }

    @Test
    fun testShowBillingPaymentAvailable() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns "$10"
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add 30 days time ($10)").assertExists()
    }

    @Test
    fun testOnPurchaseBillingProductClick() {
        // Arrange
        val clickHandler: (String) -> Unit = mockk(relaxed = true)
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns "$10"
        every { mockPaymentProduct.productId } returns "PRODUCT_ID"
        composeTestRule.setContent {
            AccountScreen(
                showSitePayment = true,
                uiState =
                    AccountUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                onPurchaseBillingProductClick = clickHandler,
                viewActions = MutableSharedFlow<AccountViewModel.ViewAction>().asSharedFlow(),
                enterTransitionEndAction = MutableSharedFlow<Unit>().asSharedFlow()
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add 30 days time ($10)").performClick()

        // Assert
        verify { clickHandler.invoke("PRODUCT_ID") }
    }

    companion object {
        private const val DUMMY_DEVICE_NAME = "fake_name"
        private const val DUMMY_ACCOUNT_NUMBER = "fake_number"
    }
}
