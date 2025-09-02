package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_NAME_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SelectLocationPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SCREEN_TEST_TAG))
    }

    fun clickLocationExpandButton(locationName: String) {
        val locationCell =
            uiDevice
                .findObjectWithTimeout(By.textContains(locationName).res(GEOLOCATION_NAME_TAG))
                .parent
                .parent
        val expandButton = locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
        expandButton.click()
    }

    fun clickLocationCell(locationName: String) {
        scrollUntilCell(locationName)
        uiDevice.findObjectWithTimeout(By.text(locationName)).click()
    }

    private fun scrollUntilCell(locationName: String) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_LIST_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(locationName)))
    }
}
