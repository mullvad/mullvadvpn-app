package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.model.TunnelState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class OutOfTimeScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                OutOfTimeScreen(
                    state = OutOfTimeUiState(deviceName = ""),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onDisconnectClick = {}
                )
            }

            // Assert
            onNodeWithText(
                    "Either buy credit on our website or redeem a voucher.",
                    substring = true
                )
                .assertDoesNotExist()
            onNodeWithText("Buy credit").assertDoesNotExist()
        }

    @Test
    fun testOpenAccountView() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                OutOfTimeScreen(
                    state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onDisconnectClick = {}
                )
            }

            // Assert
            onNodeWithText("Congrats!").assertDoesNotExist()
        }

    @Test
    fun testClickSitePaymentButton() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                OutOfTimeScreen(
                    state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                    onSitePaymentClick = mockClickListener,
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onDisconnectClick = {}
                )
            }

            // Act
            onNodeWithText("Buy credit").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testClickRedeemVoucher() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                OutOfTimeScreen(
                    state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = mockClickListener,
                    onSettingsClick = {},
                    onAccountClick = {},
                    onDisconnectClick = {}
                )
            }

            // Act
            onNodeWithText("Redeem voucher").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testClickDisconnect() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            tunnelState = TunnelState.Connecting(null, null),
                            deviceName = "",
                            showSitePayment = true
                        ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onDisconnectClick = mockClickListener
                )
            }

            // Act
            onNodeWithText("Disconnect").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testShowBillingErrorPaymentButton() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            showSitePayment = true,
                            billingPaymentState = PaymentState.Error.Billing
                        ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> }
                )
            }

            // Assert
            onNodeWithText("Add 30 days time").assertExists()
        }

    @Test
    fun testShowBillingPaymentAvailable() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns null
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            showSitePayment = true,
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
            onNodeWithText("Add 30 days time ($10)").assertExists()
        }

    @Test
    fun testShowPendingPayment() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns PaymentStatus.PENDING
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            showSitePayment = true,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                )
            }

            // Assert
            onNodeWithText("Google Play payment pending").assertExists()
        }

    @Test
    fun testShowPendingPaymentInfoDialog() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns PaymentStatus.PENDING
            val mockNavigateToVerificationPending: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            showSitePayment = true,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                    navigateToVerificationPendingDialog = mockNavigateToVerificationPending
                )
            }

            // Act
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).assertExists()

            // Assert
            verify(exactly = 1) { mockNavigateToVerificationPending.invoke() }
        }

    @Test
    fun testShowVerificationInProgress() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns PaymentStatus.VERIFICATION_IN_PROGRESS
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = true,
                        )
                )
            }

            // Assert
            onNodeWithText("Verifying purchase").assertExists()
        }

    @Test
    fun testOnPurchaseBillingProductClick() =
        composeExtension.use {
            // Arrange
            val clickHandler: (ProductId) -> Unit = mockk(relaxed = true)
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.productId } returns ProductId("PRODUCT_ID")
            every { mockPaymentProduct.status } returns null
            setContentWithTheme {
                OutOfTimeScreen(
                    state =
                        OutOfTimeUiState(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = true,
                        ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = clickHandler
                )
            }

            // Act
            onNodeWithText("Add 30 days time ($10)").performClick()

            // Assert
            verify { clickHandler(ProductId("PRODUCT_ID")) }
        }
}
