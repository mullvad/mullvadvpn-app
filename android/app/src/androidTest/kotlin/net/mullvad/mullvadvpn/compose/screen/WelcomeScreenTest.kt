package net.mullvad.mullvadvpn.compose.screen

import android.app.Activity
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.util.toPaymentDialogData
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class WelcomeScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() {
        // Arrange
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText("Congrats!").assertExists()
            onNodeWithText("Here’s your account number. Save it!").assertExists()
        }
    }

    @Test
    fun testDisableSitePayment() {
        // Arrange
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = false,
                uiState = WelcomeUiState(),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.apply {
            onNodeWithText(
                    "Either buy credit on our website or redeem a voucher.",
                    substring = true
                )
                .assertDoesNotExist()
            onNodeWithText("Buy credit").assertDoesNotExist()
        }
    }

    @Test
    fun testShowAccountNumber() {
        // Arrange
        val rawAccountNumber = "1111222233334444"
        val expectedAccountNumber = "1111 2222 3333 4444"
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(accountNumber = rawAccountNumber),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.apply { onNodeWithText(expectedAccountNumber).assertExists() }
    }

    @Test
    fun testOpenAccountView() {
        // Arrange
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(),
                uiSideEffect =
                    MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenAccountView("222")),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.apply { onNodeWithText("Congrats!").assertDoesNotExist() }
    }

    @Test
    fun testOpenConnectScreen() {
        // Arrange
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = mockClickListener,
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testClickSitePaymentButton() {
        // Arrange
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = mockClickListener,
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Act
        composeTestRule.apply { onNodeWithText("Buy credit").performClick() }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testClickRedeemVoucher() {
        // Arrange
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState(),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = mockClickListener,
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Act
        composeTestRule.apply { onNodeWithText("Redeem voucher").performClick() }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testShowPurchaseCompleteDialog() {
        // Arrange
        composeTestRule.setContent {
            WelcomeScreen(
                showSitePayment = true,
                uiState =
                    WelcomeUiState(
                        paymentDialogData = PurchaseResult.Completed.Success.toPaymentDialogData()
                    ),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Time was successfully added").assertExists()
    }

    @Test
    fun testShowVerificationErrorDialog() {
        // Arrange
        composeTestRule.setContent {
            WelcomeScreen(
                showSitePayment = true,
                uiState =
                    WelcomeUiState(
                        paymentDialogData =
                            PurchaseResult.Error.VerificationError(null).toPaymentDialogData()
                    ),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Verifying purchase").assertExists()
    }

    @Test
    fun testShowBillingErrorPaymentButton() {
        // Arrange
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                showSitePayment = true,
                uiState = WelcomeUiState().copy(billingPaymentState = PaymentState.Error.Billing),
                uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onClosePurchaseResultDialog = {},
                onPurchaseBillingProductClick = { _, _ -> }
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
        composeTestRule.setContent {
            WelcomeScreen(
                showSitePayment = true,
                uiState =
                    WelcomeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _, _ -> },
                onClosePurchaseResultDialog = {}
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Add 30 days time ($10)").assertExists()
    }

    @Test
    fun testOnPurchaseBillingProductClick() {
        // Arrange
        val clickHandler: (ProductId, () -> Activity) -> Unit = mockk(relaxed = true)
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.productId } returns ProductId("PRODUCT_ID")
        every { mockPaymentProduct.status } returns null
        composeTestRule.setContent {
            WelcomeScreen(
                showSitePayment = true,
                uiState =
                    WelcomeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = clickHandler,
                onClosePurchaseResultDialog = {}
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add 30 days time ($10)").performClick()

        // Assert
        verify { clickHandler(ProductId("PRODUCT_ID"), any()) }
    }
}
