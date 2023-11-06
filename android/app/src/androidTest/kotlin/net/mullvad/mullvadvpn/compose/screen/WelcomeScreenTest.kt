package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
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
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
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
                uiState = WelcomeUiState(),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                navigateToDeviceInfoDialog = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {}
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
                uiState = WelcomeUiState(),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                navigateToDeviceInfoDialog = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {}
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
                uiState = WelcomeUiState(accountNumber = rawAccountNumber),
                uiSideEffect = MutableSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToDeviceInfoDialog = {},
                navigateToVerificationPendingDialog = {}
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
                uiState = WelcomeUiState(),
                uiSideEffect =
                    MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenAccountView("222")),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToDeviceInfoDialog = {},
                navigateToVerificationPendingDialog = {}
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
                uiState = WelcomeUiState(),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = mockClickListener,
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
                uiState = WelcomeUiState(showSitePayment = true),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = mockClickListener,
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
                uiState = WelcomeUiState(),
                uiSideEffect = MutableStateFlow(WelcomeViewModel.UiSideEffect.OpenConnectScreen),
                onSitePaymentClick = {},
                onRedeemVoucherClick = mockClickListener,
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
            )
        }

        // Act
        composeTestRule.apply { onNodeWithText("Redeem voucher").performClick() }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testShowBillingErrorPaymentButton() {
        // Arrange
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                uiState = WelcomeUiState().copy(billingPaymentState = PaymentState.Error.Billing),
                uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
            WelcomeScreen(
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
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
            WelcomeScreen(
                uiState =
                    WelcomeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
        val mockShowPendingInfo = mockk<() -> Unit>(relaxed = true)
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                uiState =
                    WelcomeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = mockShowPendingInfo,
                navigateToDeviceInfoDialog = {}
            )
        }

        // Act
        composeTestRule.onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

        // Assert
        verify(exactly = 1) { mockShowPendingInfo() }
    }

    @Test
    fun testShowVerificationInProgress() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns PaymentStatus.VERIFICATION_IN_PROGRESS
        composeTestRule.setContentWithTheme {
            WelcomeScreen(
                uiState =
                    WelcomeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                openConnectScreen = {},
                onPurchaseBillingProductClick = { _ -> },
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
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
            WelcomeScreen(
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
                navigateToVerificationPendingDialog = {},
                navigateToDeviceInfoDialog = {}
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add 30 days time ($10)").performClick()

        // Assert
        verify { clickHandler(ProductId("PRODUCT_ID")) }
    }
}
