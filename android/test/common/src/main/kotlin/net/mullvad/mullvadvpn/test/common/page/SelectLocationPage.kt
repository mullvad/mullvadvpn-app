package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.findObjectsWithTimeout

class SelectLocationPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SCREEN_TEST_TAG))
    }

    fun clickLocationExpandButton(locationName: String) {
        val locationCells =
            uiDevice.findObjectsWithTimeout(By.textContains(locationName)).map { it.parent.parent }
        var foundAny = false
        locationCells.forEach { locationCell ->
            try {
                val expandButton =
                    locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
                expandButton.click()
                foundAny = true
            } catch (_: IllegalArgumentException) {
                // If the expand button is not found, we skip this cell
            }
        }
        require(foundAny) { "No expand button found for location: $locationName" }
    }

    fun clickLocationCell(locationName: String) {
        uiDevice.findObjectWithTimeout(By.text(locationName)).click()
    }
}
