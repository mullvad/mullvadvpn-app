package net.mullvad.mullvadvpn.compose.screen

import android.content.res.Resources
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.compose.setContentWithTheme
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.compose.test.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.VoucherRegexHelper
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class RedeemVoucherDialogTest {
    @get:Rule val composeTestRule = createComposeRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockResources: Resources = mockk()

    private val _connectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    val connectionState = _connectionState.asStateFlow()

    @Before
    fun setup() {
        MockKAnnotations.init(this)
        mockkObject(VoucherRegexHelper)
    }

    @Test
    fun testInsertInvalidVoucher() {
        // Arrange
        every { VoucherRegexHelper.validate(any()) } returns false
        every { mockServiceConnectionManager.connectionState } returns connectionState

        // Act
        val vm = VoucherDialogViewModel(mockServiceConnectionManager, mockResources)
        val uiState = vm.uiState.value
        composeTestRule.setContentWithTheme {
            RedeemVoucherDialogScreen(
                uiState = uiState,
                onVoucherInputChange = { vm.onVoucherInputChange(it) },
                onRedeem = {},
                onDismiss = {}
            )
        }

        // Sets the TextField value
        composeTestRule.onNodeWithTag(VOUCHER_INPUT_TEST_TAG).performTextInput(DUMMY_VALID_VOUCHER)
        composeTestRule.onNodeWithText(REDEEM_BUTTON_TEXT).performClick()

        // Assert
        composeTestRule.onNodeWithText(DUMMY_INVALID_VOUCHER).assertDoesNotExist()
    }

    @Test
    fun testDismissDialog() {
        // Arrange
        val mockedClickHandler: (Boolean) -> Unit = mockk(relaxed = true)

        // Act
        composeTestRule.setContentWithTheme {
            RedeemVoucherDialogScreen(
                uiState = VoucherDialogUiState.INITIAL,
                onVoucherInputChange = {},
                onRedeem = {},
                onDismiss = mockedClickHandler
            )
        }

        composeTestRule.onNodeWithText(CANCEL_BUTTON_TEXT).performClick()

        // Assert
        verify { mockedClickHandler.invoke(false) }
    }

    companion object {
        private const val REDEEM_BUTTON_TEXT = "Redeem"
        private const val CANCEL_BUTTON_TEXT = "Cancel"
        //        private const val DUMMY_VALID_VOUCHER = "DUMM-YVAL-IDVO-UCHE-R"
        private const val DUMMY_VALID_VOUCHER = "DUMMYVALIDVOUCHER"
        private const val DUMMY_USED_VOUCHER = "DUMM-YUSE-DVOU-CHER"
        private const val DUMMY_INVALID_VOUCHER = "DUMM-YINV-ALID-VOUC-HER"
    }
}
