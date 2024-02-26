package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.dialog.RedeemVoucherDialog
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.test.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class RedeemVoucherDialogTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @BeforeEach
    fun setup() {
        mockkObject(VoucherRegexHelper)
    }

    @Test
    fun testDismissDialog() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState = VoucherDialogUiState.INITIAL,
                    onVoucherInputChange = {},
                    onRedeem = {},
                    onDismiss = mockedClickHandler
                )
            }

            // Act
            onNodeWithText(CANCEL_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedClickHandler.invoke(false) }
        }

    @Test
    fun testDismissDialogAfterSuccessfulRedeem() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState =
                        VoucherDialogUiState(voucherViewModelState = VoucherDialogState.Success(0)),
                    onVoucherInputChange = {},
                    onRedeem = {},
                    onDismiss = mockedClickHandler
                )
            }

            // Act
            onNodeWithText(GOT_IT_BUTTON_TEXT).performClick()

            // Assert
            verify { mockedClickHandler.invoke(true) }
        }

    @Test
    fun testInsertVoucher() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (String) -> Unit = mockk(relaxed = true)
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState = VoucherDialogUiState(),
                    onVoucherInputChange = mockedClickHandler,
                    onRedeem = {},
                    onDismiss = {}
                )
            }

            // Act
            onNodeWithTag(VOUCHER_INPUT_TEST_TAG).performTextInput(DUMMY_VOUCHER)

            // Assert
            verify { mockedClickHandler.invoke(DUMMY_VOUCHER) }
        }

    @Test
    fun testVerifyingState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState =
                        VoucherDialogUiState(voucherViewModelState = VoucherDialogState.Verifying),
                    onVoucherInputChange = {},
                    onRedeem = {},
                    onDismiss = {}
                )
            }

            // Assert
            onNodeWithText("Verifying voucher…").assertExists()
        }

    @Test
    fun testSuccessState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState =
                        VoucherDialogUiState(voucherViewModelState = VoucherDialogState.Success(0)),
                    onVoucherInputChange = {},
                    onRedeem = {},
                    onDismiss = {}
                )
            }

            // Assert
            onNodeWithText("Voucher was successfully redeemed.").assertExists()
        }

    @Test
    fun testErrorState() =
        composeExtension.use {
            // Arrange
            setContentWithTheme {
                RedeemVoucherDialog(
                    uiState =
                        VoucherDialogUiState(
                            voucherViewModelState = VoucherDialogState.Error(ERROR_MESSAGE)
                        ),
                    onVoucherInputChange = {},
                    onRedeem = {},
                    onDismiss = {}
                )
            }

            // Assert
            onNodeWithText(ERROR_MESSAGE).assertExists()
        }

    companion object {
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val GOT_IT_BUTTON_TEXT = "Got it!"
        private const val DUMMY_VOUCHER = "DUMMY____VOUCHER"
        private const val ERROR_MESSAGE = "error_message"
    }
}
