package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AccountPage internal constructor() : Page() {
    private val logOutSelector = By.text("Log out")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.text("Account"))
    }

    fun clickLogOut() {
        uiDevice.findObjectWithTimeout(logOutSelector).click()
    }
}
