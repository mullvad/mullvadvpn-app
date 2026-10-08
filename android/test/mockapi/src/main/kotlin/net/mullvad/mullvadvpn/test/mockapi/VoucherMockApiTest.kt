package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.lib.ui.tag.CONNECT_CARD_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.AddTimeBottomSheet
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.RedeemVoucherDialog
import net.mullvad.mullvadvpn.test.common.page.WelcomePage
import net.mullvad.mullvadvpn.test.common.page.dismissStorePasswordPromptIfShown
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Test

class VoucherMockApiTest : MockApiTest() {

    @Test
    fun testCreateAccountAndRedeemVoucher() {
        // Arrange
        val createdAccountNumber = "1234123412341234"
        val voucherCode = "AA1AB2BBC3CCD45D"
        apiRouter.apply {
            expectedAccountNumber = createdAccountNumber
            expectedVoucherCode = voucherCode
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> { clickCreateAccount() }

        device.dismissStorePasswordPromptIfShown()

        on<WelcomePage> { clickAddTime() }

        on<AddTimeBottomSheet> { clickRedeemVoucher() }

        apiRouter.accountExpiry = ZonedDateTime.now().plusMonths(1)
        apiRouter.hasPayments = true

        on<RedeemVoucherDialog> {
            enterVoucherCode(voucherCode)
            clickRedeemButton()
            assertSuccessful()
            clickGotItButton()
        }

        app.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        // Assert we reach the Connect page after redeeming voucher.
        // The timeout here is the default since we will update the expiry date and has payments
        // directly from the redeem voucher response, so we don't need to wait for the account to be
        // polled from the api.
        device.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG))
    }
}
