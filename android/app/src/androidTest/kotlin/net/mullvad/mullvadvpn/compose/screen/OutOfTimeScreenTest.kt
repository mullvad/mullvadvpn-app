package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.model.TunnelState
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class OutOfTimeScreenTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDisableSitePayment() {
        // Arrange
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = false,
                uiState = OutOfTimeUiState(deviceName = ""),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onDisconnectClick = {}
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
    fun testOpenAccountView() {
        // Arrange
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = true,
                uiState = OutOfTimeUiState(deviceName = ""),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onDisconnectClick = {}
            )
        }

        // Assert
        composeTestRule.apply { onNodeWithText("Congrats!").assertDoesNotExist() }
    }

    @Test
    fun testClickSitePaymentButton() {
        // Arrange
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = true,
                uiState = OutOfTimeUiState(deviceName = ""),
                onSitePaymentClick = mockClickListener,
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onDisconnectClick = {}
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
            OutOfTimeScreen(
                showSitePayment = true,
                uiState = OutOfTimeUiState(deviceName = ""),
                onSitePaymentClick = {},
                onRedeemVoucherClick = mockClickListener,
                onSettingsClick = {},
                onAccountClick = {},
                onDisconnectClick = {}
            )
        }

        // Act
        composeTestRule.apply { onNodeWithText("Redeem voucher").performClick() }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testClickDisconnect() {
        // Arrange
        val mockClickListener: () -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState(
                        tunnelState = TunnelState.Connecting(null, null),
                        deviceName = ""
                    ),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onDisconnectClick = mockClickListener
            )
        }

        // Act
        composeTestRule.apply { onNodeWithText("Disconnect").performClick() }

        // Assert
        verify(exactly = 1) { mockClickListener.invoke() }
    }

    @Test
    fun testShowBillingErrorPaymentButton() {
        // Arrange
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = true,
                uiState = OutOfTimeUiState().copy(billingPaymentState = PaymentState.Error.Billing),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onPurchaseBillingProductClick = { _ -> }
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
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onPurchaseBillingProductClick = { _ -> }
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
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
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
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                navigateToVerificationPendingDialog = mockNavigateToVerificationPending
            )
        }

        // Act
        composeTestRule.onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()
        composeTestRule.onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).assertExists()

        verify(exactly = 1) { mockNavigateToVerificationPending.invoke() }
    }

    @Test
    fun testShowVerificationInProgress() {
        // Arrange
        val mockPaymentProduct: PaymentProduct = mockk()
        every { mockPaymentProduct.price } returns ProductPrice("$10")
        every { mockPaymentProduct.status } returns PaymentStatus.VERIFICATION_IN_PROGRESS
        composeTestRule.setContentWithTheme {
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState()
                        .copy(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
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
            OutOfTimeScreen(
                showSitePayment = true,
                uiState =
                    OutOfTimeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                    ),
                onSitePaymentClick = {},
                onRedeemVoucherClick = {},
                onSettingsClick = {},
                onAccountClick = {},
                onPurchaseBillingProductClick = clickHandler
            )
        }

        // Act
        composeTestRule.onNodeWithText("Add 30 days time ($10)").performClick()

        // Assert
        verify { clickHandler(ProductId("PRODUCT_ID")) }
    }
}
