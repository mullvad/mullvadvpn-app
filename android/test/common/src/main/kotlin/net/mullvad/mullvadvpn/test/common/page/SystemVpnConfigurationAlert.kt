package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SystemVpnConfigurationAlert internal constructor() : Page() {
    private val byOkText = By.text("OK")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(byOkText)
    }

    fun clickOk() {
        uiDevice.findObjectWithTimeout(byOkText).click()
    }
}
