package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class MultihopSettingsPage internal constructor() : Page() {

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(MULTIHOP_SCREEN_TEST_TAG))
    }

    fun clickEnableMultihopAlways() {
        uiDevice.findObjectWithTimeout(By.text("Always")).click()
    }
}
