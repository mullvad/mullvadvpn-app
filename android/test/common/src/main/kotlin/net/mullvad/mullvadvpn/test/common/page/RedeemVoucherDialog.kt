package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.VOUCHER_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class RedeemVoucherDialog internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.text("Enter voucher code"))
    }

    fun enterVoucherCode(voucherCode: String) {
        val input = uiDevice.findObjectWithTimeout(By.res(VOUCHER_INPUT_TEST_TAG))
        input.text = voucherCode
    }

    fun clickRedeemButton() {
        uiDevice.findObjectWithTimeout(By.text("Redeem")).click()
    }

    fun assertSuccessful() {
        uiDevice.findObjectWithTimeout(By.text("Voucher was successfully redeemed."))
    }

    fun clickGotItButton() {
        uiDevice.findObjectWithTimeout(By.text("Got it!")).click()
    }
}
