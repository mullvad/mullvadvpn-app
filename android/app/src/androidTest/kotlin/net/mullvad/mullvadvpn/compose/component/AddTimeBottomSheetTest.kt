package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SheetState
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlin.Unit
import net.mullvad.mullvadvpn.compose.bottomsheet.addtime.AddTimeBottomSheetContent
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.core.toLc
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
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
                positionalThreshold = { 0f },
                velocityThreshold = { 0f },
            ),
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
                            billingPaymentState = PaymentState.NoPayment,
                            showSitePayment = true,
                            tunnelStateBlocked = false,
                        )
                        .toLc(),
                onSitePaymentClick = mockedClickHandler,
            )

            // Act
            onNodeWithText(BUY_CREDIT_TEXT).performClick()

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
                            billingPaymentState = PaymentState.NoPayment,
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
                            tunnelStateBlocked = false,
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
            val productId = ProductId(ProductIds.ThreeMonths)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = PurchaseState.Success(productId),
                            billingPaymentState =
                                PaymentState.PaymentAvailable(
                                    listOf(
                                        PaymentProduct(
                                            productId = productId,
                                            price = ProductPrice("$30"),
                                            status = null,
                                        )
                                    )
                                ),
                            tunnelStateBlocked = false,
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
            val productId = ProductId(ProductIds.ThreeMonths)
            initBottomSheet(
                AddTimeUiState(
                        purchaseState = PurchaseState.VerifyingPurchase,
                        billingPaymentState =
                            PaymentState.PaymentAvailable(
                                listOf(
                                    PaymentProduct(
                                        productId = productId,
                                        price = ProductPrice("$30"),
                                        status = null,
                                    )
                                )
                            ),
                        tunnelStateBlocked = false,
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

            val productId = ProductId(ProductIds.ThreeMonths)
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = PurchaseState.Error.OtherError(productId),
                            billingPaymentState =
                                PaymentState.PaymentAvailable(
                                    listOf(
                                        PaymentProduct(
                                            productId = productId,
                                            price = ProductPrice("$30"),
                                            status = null,
                                        )
                                    )
                                ),
                            tunnelStateBlocked = false,
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
                            billingPaymentState = PaymentState.NoPayment,
                            tunnelStateBlocked = false,
                            showSitePayment = false,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText(BUY_CREDIT_TEXT).assertDoesNotExist()
        }

    @Test
    fun testShowInternetBlocked() =
        composeExtension.use {
            // Arrange
            initBottomSheet(
                state =
                    AddTimeUiState(
                            purchaseState = null,
                            billingPaymentState = PaymentState.NoPayment,
                            tunnelStateBlocked = true,
                            showSitePayment = true,
                        )
                        .toLc()
            )

            // Assert
            onNodeWithText("The app is blocking internet, please disconnect first").assertExists()
        }

    companion object {
        private const val BUY_CREDIT_TEXT = "Buy credit"
    }
}
