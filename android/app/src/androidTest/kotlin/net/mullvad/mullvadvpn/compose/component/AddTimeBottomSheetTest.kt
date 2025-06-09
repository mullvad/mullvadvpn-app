package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SheetState
import androidx.compose.material3.SheetValue
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.unit.Density
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlin.Unit
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalMaterial3Api::class)
class AddTimeBottomSheetTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initBottomSheet(
        state: Lc<Unit, AddTimeUiState> = Lc.Loading(Unit),
        sheetState: SheetState =
            SheetState(
                skipPartiallyExpanded = true,
                density = Density(1f),
                initialValue = SheetValue.Expanded,
            ),
        internetBlocked: Boolean = false,
        onPurchaseBillingProductClick: (ProductId) -> Unit = {},
        onPlayPaymentInfoClick: () -> Unit = {},
        onSitePaymentClick: () -> Unit = {},
        onRedeemVoucherClick: () -> Unit = {},
        onRetryFetchProducts: () -> Unit = {},
        resetPurchaseState: () -> Unit = {},
        closeSheetAndResetPurchaseState: (Boolean) -> Unit = {},
        closeBottomSheet: (animate: Boolean) -> Unit = {},
    ) {
        setContentWithTheme {
            AddTimeBottomSheetContent(
                state = state,
                sheetState = sheetState,
                internetBlocked = internetBlocked,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onPlayPaymentInfoClick = onPlayPaymentInfoClick,
                onSitePaymentClick = onSitePaymentClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onRetryFetchProducts = onRetryFetchProducts,
                resetPurchaseState = resetPurchaseState,
                closeSheetAndResetPurchaseState = closeSheetAndResetPurchaseState,
                closeBottomSheet = closeBottomSheet,
            )
        }
    }

    @Test
    fun testBuyCreditClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = null,
                            showSitePayment = true,
                        )
                        .toLc(),
                onSitePaymentClick = mockedClickHandler,
            )

            // Act
            onNodeWithText("Buy time from website").performClick()

            // Assert
            verify(exactly = 1) { mockedClickHandler.invoke() }
        }

    @Test
    fun testRedeemVoucherClick() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: () -> Unit = mockk(relaxed = true)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = null,
                            showSitePayment = true,
                        )
                        .toLc(),
                onRedeemVoucherClick = mockedClickHandler,
            )

            // Act
            onNodeWithText("Redeem voucher").performClick()

            // Assert
            verify { mockedClickHandler.invoke() }
        }

    @Test
    fun testShowBillingErrorPaymentButton() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = PaymentState.Error.Generic,
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("Failed to load products, please try again").assertExists()
        }

    @Test
    fun testShowBillingPaymentAvailable() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.productId } returns ProductId(ProductIds.OneMonth)
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns null
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = false,
                        )
                        .toLc()
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
            every { mockPaymentProduct.productId } returns ProductId(ProductIds.OneMonth)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("Google Play payment pending, this might take some time").assertExists()
        }

    @Test
    fun testShowPendingPaymentInfoDialog() =
        composeExtension.use {
            // Arrange
            val mockPaymentProduct: PaymentProduct = mockk()
            every { mockPaymentProduct.price } returns ProductPrice("$10")
            every { mockPaymentProduct.status } returns PaymentStatus.PENDING
            every { mockPaymentProduct.productId } returns ProductId(ProductIds.OneMonth)
            val mockNavigateToVerificationPending: () -> Unit = mockk(relaxed = true)
            PaymentState.PaymentAvailable(listOf(mockPaymentProduct))
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = false,
                        )
                        .toLc(),
                onPlayPaymentInfoClick = mockNavigateToVerificationPending,
            )

            // Act
            onNodeWithTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG).performClick()

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
            every { mockPaymentProduct.productId } returns ProductId(ProductIds.ThreeMonths)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = false,
                        )
                        .toLc()
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
            every { mockPaymentProduct.productId } returns ProductId(ProductIds.OneMonth)
            every { mockPaymentProduct.status } returns null
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState =
                                PaymentState.PaymentAvailable(listOf(mockPaymentProduct)),
                            showSitePayment = false,
                        )
                        .toLc(),
                onPurchaseBillingProductClick = clickHandler,
            )

            // Act
            onNodeWithText("Add 30 days time ($10)").performClick()

            // Assert
            verify { clickHandler.invoke(ProductId(ProductIds.OneMonth)) }
        }

    @Test
    fun testShowPurchaseCompleteDialog() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState =
                                PurchaseState.Success(ProductId(ProductIds.ThreeMonths)),
                            billingPaymentState = null,
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("Time added").assertExists()
            onNodeWithText("90 days was added to your account.").assertExists()
        }

    @Test
    fun testShowVerificationErrorDialog() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                AddTimeUiState(
                        purchaseState = PurchaseState.VerifyingPurchase,
                        billingPaymentState = null,
                        showSitePayment = false,
                    )
                    .toLc()
            )

            // Assert
            onNodeWithText("Verifying purchase").assertExists()
        }

    @Test
    fun testShowFetchProductsErrorDialog() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = PurchaseState.Error.OtherError(ProductId("ProductId")),
                            billingPaymentState = null,
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText(
                    "We were unable to start the payment process, please make sure you have the latest version of Google Play."
                )
                .assertExists()
        }

    @Test
    fun testDisableSitePayment() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = null,
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("Buy time from website").assertDoesNotExist()
        }

    @Test
    fun testShowInternetBlocked() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = null,
                            showSitePayment = true,
                        )
                        .toLc(),
                internetBlocked = true,
            )

            // Assert
            onNodeWithText("The app is blocking internet, please disconnect first").assertExists()
        }
}
