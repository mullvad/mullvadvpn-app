package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import de.mannodermaus.junit5.compose.ComposeContext
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.test.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherError
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

    private fun ComposeContext.initDialog(
        state: VoucherDialogUiState = VoucherDialogUiState.INITIAL,
        onVoucherInputChange: (String) -> Unit = {},
        onRedeem: (voucherCode: String) -> Unit = {},
        onDismiss: (isTimeAdded: Boolean) -> Unit = {},
    ) {
        setContentWithTheme {
            RedeemVoucherDialog(
                state = state,
                onVoucherInputChange = onVoucherInputChange,
                onRedeem = onRedeem,
                onDismiss = onDismiss,
            )
        }
    }

    @Test
    fun testDismissDialog() =
        composeExtension.use {
            // Arrange
            val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)
            initDialog(onDismiss = mockedClickHandler)

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
            initDialog(
                state = VoucherDialogUiState(voucherState = VoucherDialogState.Success(0)),
                onDismiss = mockedClickHandler,
            )

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
            initDialog(state = VoucherDialogUiState(), onVoucherInputChange = mockedClickHandler)

            // Act
            onNodeWithTag(VOUCHER_INPUT_TEST_TAG).performTextInput(DUMMY_VOUCHER)

            // Assert
            verify { mockedClickHandler.invoke(DUMMY_VOUCHER) }
        }

    @Test
    fun testVerifyingState() =
        composeExtension.use {
            // Arrange
            initDialog(state = VoucherDialogUiState(voucherState = VoucherDialogState.Verifying))

            // Assert
            onNodeWithText("Verifying voucher…").assertExists()
        }

    @Test
    fun testSuccessState() =
        composeExtension.use {
            // Arrange
            initDialog(state = VoucherDialogUiState(voucherState = VoucherDialogState.Success(0)))

            // Assert
            onNodeWithText("Voucher was successfully redeemed.").assertExists()
        }

    @Test
    fun testErrorState() =
        composeExtension.use {
            // Arrange
            initDialog(
                state =
                    VoucherDialogUiState(
                        voucherState = VoucherDialogState.Error(RedeemVoucherError.InvalidVoucher)
                    )
            )

            // Assert
            onNodeWithText(VOUCHER_CODE_INVALID_ERROR_MESSAGE).assertExists()
        }

    companion object {
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val GOT_IT_BUTTON_TEXT = "Got it!"
        private const val DUMMY_VOUCHER = "DUMMY____VOUCHER"
        private const val VOUCHER_CODE_INVALID_ERROR_MESSAGE = "Voucher code is invalid."
    }
}
