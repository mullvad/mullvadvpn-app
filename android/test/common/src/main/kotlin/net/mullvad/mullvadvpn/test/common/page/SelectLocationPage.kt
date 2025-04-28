package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG

class SelectLocationPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SCREEN_TEST_TAG))
    }

    fun clickLocationExpandButton(locationName: String) {
        val locationCell = uiDevice.findObjectWithTimeout(By.text(locationName)).parent.parent
        val expandButton = locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
        expandButton.click()
    }

    fun clickLocationCell(locationName: String) {
        uiDevice.findObjectWithTimeout(By.text(locationName)).click()
    }
}
