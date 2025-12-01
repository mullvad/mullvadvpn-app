package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class MultihopSettingsPage internal constructor() : Page() {
    private val enableSelector = By.text("Enable")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(MULTIHOP_SCREEN_TEST_TAG))
    }

    fun clickEnableSwitch() {
        val enableCell = uiDevice.findObjectWithTimeout(enableSelector).parent
        val enableSwitch = enableCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        enableSwitch.click()
    }
}
