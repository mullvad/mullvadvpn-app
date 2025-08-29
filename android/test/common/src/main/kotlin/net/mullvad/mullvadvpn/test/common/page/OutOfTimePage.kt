package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.OUT_OF_TIME_SCREEN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class OutOfTimePage internal constructor() : Page() {
    private val outOfTimeSelector = By.res(OUT_OF_TIME_SCREEN_TITLE_TEST_TAG)

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(outOfTimeSelector)
    }

    fun clickAddTime() {
        uiDevice.findObjectWithTimeout(By.text("Add time")).click()
    }
}
