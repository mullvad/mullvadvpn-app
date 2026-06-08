package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SEARCH_LOCATION_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SEARCH_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SearchLocationPage : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SEARCH_LOCATION_SCREEN_TEST_TAG))
    }

    fun searchInput(input: String) {
        uiDevice.findObjectWithTimeout(By.res(SEARCH_LOCATION_INPUT_TEST_TAG)).text = input
    }

    fun clickLocation(locationName: String) {
        uiDevice
            .findObjectWithTimeout(By.text(locationName).hasAncestor(By.res(GEOLOCATION_ITEM_TAG)))
            .click()
    }
}
