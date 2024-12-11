package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.test.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
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

    private fun ComposeContext.initScreen(
        state: OutOfTimeUiState = OutOfTimeUiState(),
        onDisconnectClick: () -> Unit = {},
        onSitePaymentClick: () -> Unit = {},
        onRedeemVoucherClick: () -> Unit = {},
        onSettingsClick: () -> Unit = {},
        onAccountClick: () -> Unit = {},
        onPurchaseBillingProductClick: (ProductId) -> Unit = {},
        navigateToVerificationPendingDialog: () -> Unit = {},
    ) {

        setContentWithTheme {
            OutOfTimeScreen(
                state = state,
                onDisconnectClick = onDisconnectClick,
                onSitePaymentClick = onSitePaymentClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onSettingsClick = onSettingsClick,
                onAccountClick = onAccountClick,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                navigateToVerificationPendingDialog = navigateToVerificationPendingDialog,
            )
        }
    }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            initScreen(state = OutOfTimeUiState(deviceName = ""))

            // Assert
            onNodeWithText(
                    "Either buy credit on our website or redeem a voucher.",
                    substring = true,
                )
                .assertDoesNotExist()
            onNodeWithText("Buy credit").assertDoesNotExist()
        }

    @Test
    fun testOpenAccountView() =
        composeExtension.use {
            val mockClickListener: () -> Unit = mockk(relaxed = true)

            // Arrange
            initScreen(
                state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                onAccountClick = mockClickListener,
            )

            onNodeWithContentDescription(label = "Account").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testClickSitePaymentButton() =
        composeExtension.use {
            // Arrange
            val mockClickListener: () -> Unit = mockk(relaxed = true)
            initScreen(
                state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                onSitePaymentClick = mockClickListener,
            )

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
            initScreen(
                state = OutOfTimeUiState(deviceName = "", showSitePayment = true),
                onRedeemVoucherClick = mockClickListener,
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        tunnelState = TunnelState.Connecting(null, null, emptyList()),
                        deviceName = "",
                        showSitePayment = true,
                    ),
                onDisconnectClick = mockClickListener,
            )

            // Act
            onNodeWithText("Disconnect").performClick()

            // Assert
            verify(exactly = 1) { mockClickListener.invoke() }
        }

    @Test
    fun testShowBillingErrorPaymentButton() =
        composeExtension.use {
            // Arrange
            initScreen(
                state =
                    OutOfTimeUiState(
                        showSitePayment = true,
                        billingPaymentState = PaymentState.Error.Billing,
                    )
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        showSitePayment = true,
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                    )
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        showSitePayment = true,
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                    )
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        showSitePayment = true,
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                    ),
                navigateToVerificationPendingDialog = mockNavigateToVerificationPending,
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                        showSitePayment = true,
                    )
            )

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
            initScreen(
                state =
                    OutOfTimeUiState(
                        billingPaymentState =
                            PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                        showSitePayment = true,
                    ),
                onPurchaseBillingProductClick = clickHandler,
            )

            // Act
            onNodeWithText("Add 30 days time ($10)").performClick()

            // Assert
            verify { clickHandler(ProductId("PRODUCT_ID")) }
        }
}
