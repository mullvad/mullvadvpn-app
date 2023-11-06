package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import io.mockk.MockKAnnotations
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialog
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.util.toPaymentDialogData
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class PaymentDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
    }

    @Test
    fun testShowPurchaseCompleteDialog() {
        // Arrange
        composeTestRule.setContentWithTheme {
            PaymentDialog(
                paymentDialogData = PurchaseResult.Completed.Success.toPaymentDialogData()!!
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Time was successfully added").assertExists()
    }

    @Test
    fun testShowVerificationErrorDialog() {
        // Arrange
        composeTestRule.setContentWithTheme {
            PaymentDialog(
                paymentDialogData =
                    PurchaseResult.Error.VerificationError(null).toPaymentDialogData()!!
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Verifying purchase").assertExists()
    }

    @Test
    fun testShowFetchProductsErrorDialog() {
        // Arrange
        composeTestRule.setContentWithTheme {
            PaymentDialog(
                paymentDialogData =
                    PurchaseResult.Error.FetchProductsError(ProductId(""), null)
                        .toPaymentDialogData()!!
            )
        }

        // Assert
        composeTestRule.onNodeWithText("Google Play unavailable").assertExists()
    }
}
