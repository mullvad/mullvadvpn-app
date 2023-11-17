package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.remember
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isNotEnabled
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.verify
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.test.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class RedeemVoucherDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    private var uiState = VoucherDialogUiState.INITIAL

    @Before
    fun setup() {
        mockkObject(VoucherRegexHelper)
    }

    @Test
    fun testDismissDialog() {
        // Arrange
        val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)
        composeTestRule.setContentWithTheme {
            RedeemVoucherDialogScreen(
                uiState = uiState,
                onVoucherInputChange = {},
                onRedeem = {},
                onDismiss = mockedClickHandler
            )
        }

        // Act

        composeTestRule.onNodeWithText(CANCEL_BUTTON_TEXT).performClick()

        // Assert
        verify { mockedClickHandler.invoke(false) }
    }

    @Test
    fun testInsertValidVoucher() {
        // Arrange
        composeTestRule.setContentWithTheme {
            var voucherInput = remember { "" }
            RedeemVoucherDialogScreen(
                uiState = uiState.copy(voucherInput = voucherInput),
                onVoucherInputChange = { voucherInput = it },
                onRedeem = {},
                onDismiss = {}
            )
        }

        // Act
        composeTestRule.onNodeWithTag(VOUCHER_INPUT_TEST_TAG).performTextInput(DUMMY_VALID_VOUCHER)

        // Assert
        composeTestRule
            .onNodeWithTag(VOUCHER_INPUT_TEST_TAG)
            .assert(hasText(DUMMY_VALID_VOUCHER_FORMATED))
    }

    @Test
    fun testInsertInvalidVoucher() {
        // Arrange
        composeTestRule.setContentWithTheme {
            var voucherInput = remember { "" }
            RedeemVoucherDialogScreen(
                uiState = uiState.copy(voucherInput = voucherInput),
                onVoucherInputChange = { voucherInput = it },
                onRedeem = {},
                onDismiss = {}
            )
        }

        // Act
        composeTestRule
            .onNodeWithTag(VOUCHER_INPUT_TEST_TAG)
            .performTextInput(DUMMY_INVALID_VOUCHER)

        // Assert
        composeTestRule.onNodeWithText(REDEEM_BUTTON_TEXT).assert(isNotEnabled())
    }

    companion object {
        private const val REDEEM_BUTTON_TEXT = "Redeem"
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        private const val DUMMY_VALID_VOUCHER = "DUMMYVALIDVOUCHE"
        private const val DUMMY_VALID_VOUCHER_FORMATED = "DUMM-YVAL-IDVO-UCHE"
        private const val DUMMY_INVALID_VOUCHER = "DUMM-YINV-ALID-VOUC-HER"
    }
}
