package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.ADD_TIME_BOTTOM_SHEET_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AddTimeBottomSheet internal constructor() : Page() {
    private val oneMonthSelector = By.textStartsWith("Add 30 days time")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(ADD_TIME_BOTTOM_SHEET_TITLE_TEST_TAG))
    }

    fun click30days() {
        uiDevice.findObjectWithTimeout(oneMonthSelector).click()
    }
}
