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
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class WelcomeScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testDefaultState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                WelcomeScreen(
                    state = WelcomeUiState(),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
                )
            }

            // Assert
            onNodeWithText("Congrats!").assertExists()
            onNodeWithText("Here’s your account number. Save it!").assertExists()
        }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                WelcomeScreen(
                    state = WelcomeUiState(),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
    fun testShowAccountNumber() =
        composeExtension.use {
            // Arrange
            val rawAccountNumber = "1111222233334444"
            val expectedAccountNumber = "1111 2222 3333 4444"
            setContentWithTheme {
                WelcomeScreen(
                    state = WelcomeUiState(accountNumber = rawAccountNumber),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
                )
            }

            // Assert
            onNodeWithText(expectedAccountNumber).assertExists()
        }

    @Test
    fun testClickSitePaymentButton() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                WelcomeScreen(
                    state = WelcomeUiState(showSitePayment = true),
                    onSitePaymentClick = mockClickListener,
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
                WelcomeScreen(
                    state = WelcomeUiState(),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = mockClickListener,
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
                )
            }

            // Act
            onNodeWithText("Redeem voucher").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testShowBillingErrorPaymentButton() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                WelcomeScreen(
                    state = WelcomeUiState().copy(billingPaymentState = PaymentState.Error.Billing),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
                WelcomeScreen(
                    state =
                        WelcomeUiState(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
                WelcomeScreen(
                    state =
                        WelcomeUiState()
                            .copy(
                                billingPaymentState =
                                    PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                            ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
            val mockShowPendingInfo = mockk<() -> Unit>(relaxed = true)
            setContentWithTheme {
                WelcomeScreen(
                    state =
                        WelcomeUiState()
                            .copy(
                                billingPaymentState =
                                    PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                            ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToVerificationPendingDialog = mockShowPendingInfo,
                    navigateToDeviceInfoDialog = {}
                )
            }

            // Act
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

            // Assert
            verify(exactly = 1) { mockShowPendingInfo() }
        }

    @Test
    fun testShowVerificationInProgress() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns PaymentStatus.VERIFICATION_IN_PROGRESS
            setContentWithTheme {
                WelcomeScreen(
                    state =
                        WelcomeUiState()
                            .copy(
                                billingPaymentState =
                                    PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                            ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = { _ -> },
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
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
                WelcomeScreen(
                    state =
                        WelcomeUiState(
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
                        ),
                    onSitePaymentClick = {},
                    onRedeemVoucherClick = {},
                    onSettingsClick = {},
                    onAccountClick = {},
                    onPurchaseBillingProductClick = clickHandler,
                    navigateToDeviceInfoDialog = {},
                    navigateToVerificationPendingDialog = {}
                )
            }

            // Act
            onNodeWithText("Add 30 days time ($10)").performClick()

            // Assert
            verify { clickHandler(ProductId("PRODUCT_ID")) }
        }
}
