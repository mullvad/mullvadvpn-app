package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class GooglePlayPaymentPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.text("1-tap buy"), LONG_TIMEOUT)
    }

    fun clickBuy() {
        uiDevice.findObjectWithTimeout(By.text("1-tap buy")).click()
    }
}
