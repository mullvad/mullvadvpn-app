package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialog
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.util.toPaymentDialogData
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class PaymentDialogTest {
    @OptIn(ExperimentalTestApi::class)
    @JvmField
    @RegisterExtension
    val composeExtension = createEdgeToEdgeComposeExtension()

    @Test
    fun testShowPurchaseCompleteDialog() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                PaymentDialog(
                    paymentDialogData = PurchaseResult.Completed.Success.toPaymentDialogData()!!
                )
            }

            // Assert
            onNodeWithText("Time was successfully added").assertExists()
        }

    @Test
    fun testShowVerificationErrorDialog() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                PaymentDialog(
                    paymentDialogData =
                        PurchaseResult.Error.VerificationError(null).toPaymentDialogData()!!
                )
            }

            // Assert
            onNodeWithText("Verifying purchase").assertExists()
        }

    @Test
    fun testShowFetchProductsErrorDialog() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                PaymentDialog(
                    paymentDialogData =
                        PurchaseResult.Error.FetchProductsError(ProductId(""), null)
                            .toPaymentDialogData()!!
                )
            }

            // Assert
            onNodeWithText("Google Play unavailable").assertExists()
        }
}
