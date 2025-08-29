package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.MANAGE_DEVICES_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AccountPage internal constructor() : Page() {
    private val logOutSelector = By.text("Log out")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.text("Account"))
    }

    fun clickManageDevices() {
        uiDevice.findObject(By.res(MANAGE_DEVICES_BUTTON_TEST_TAG)).click()
    }

    fun clickLogOut() {
        uiDevice.findObjectWithTimeout(logOutSelector).click()
    }
}
