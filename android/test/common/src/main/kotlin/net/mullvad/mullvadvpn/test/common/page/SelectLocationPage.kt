package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

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

    companion object {
        const val SELECT_LOCATION_SCREEN_TEST_TAG = "select_location_screen_test_tag"
        const val EXPAND_BUTTON_TEST_TAG = "expand_button_test_tag"
    }
}
