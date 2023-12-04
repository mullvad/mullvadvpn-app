package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
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
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null,
                        showSitePayment = false
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
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
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        showSitePayment = true,
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null,
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
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
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null,
                        showSitePayment = false
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
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
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState(
                        deviceName = DUMMY_DEVICE_NAME,
                        accountNumber = DUMMY_ACCOUNT_NUMBER,
                        accountExpiry = null,
                        showSitePayment = false
                    ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                onLogoutClick = mockedClickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Log out").performClick()

        // Assert
        verify { mockedClickHandler.invoke() }
    }

    @Test
    fun testShowBillingErrorPaymentButton() {
        // Arrange
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default().copy(billingPaymentState = PaymentState.Error.Billing),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add 30 days time").assertExists()
    }

    @Test
    fun testShowBillingPaymentAvailable() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns null
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add 30 days time ($10)").assertExists()
    }

    @Test
    fun testShowPendingPayment() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns PaymentStatus.PENDING
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Google Play payment pending").assertExists()
    }

    @Test
    fun testShowPendingPaymentInfoDialog() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns PaymentStatus.PENDING
        val mockNavigateToVerificationPending: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
                navigateToVerificationPendingDialog = mockNavigateToVerificationPending
            )
        }

        // Act
        composeTestRule.onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

        // Assert
        verify(exactly = 1) { mockNavigateToVerificationPending.invoke() }
    }

    @Test
    fun testShowVerificationInProgress() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns PaymentStatus.VERIFICATION_IN_PROGRESS
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Verifying purchase").assertExists()
    }

    @Test
    fun testOnPurchaseBillingProductClick() {
        // Arrange
        val clickHandler: (ProductId) -> Unit = mockk(relaxed = true)
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.productId } returns ProductId("PRODUCT_ID")
        every { mockPaymentProduct.status } returns null
        composeTestRule.setContentWithTheme {
            AccountScreen(
                uiState =
                    AccountUiState.default()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                onPurchaseBillingProductClick = clickHandler,
                uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add 30 days time ($10)").performClick()

        // Assert
        verify { clickHandler.invoke(ProductId("PRODUCT_ID")) }
    }

    companion object {
        private const val DUMMY_DEVICE_NAME = "fake_name"
        private const val DUMMY_ACCOUNT_NUMBER = "fake_number"
    }
}
