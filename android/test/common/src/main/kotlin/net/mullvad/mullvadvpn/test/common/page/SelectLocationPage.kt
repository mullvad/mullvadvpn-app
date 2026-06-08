package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.HOP_SELECTOR_ENTRY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_MENU_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SEARCH_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SelectLocationPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SCREEN_TEST_TAG))
    }

    fun scrollUntilText(text: String, direction: Direction = Direction.DOWN) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_LIST_TEST_TAG))
        scrollView2.scrollUntil(direction, Until.hasObject(By.text(text)))
    }

    fun clickMenuButton() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_MENU_BUTTON_TEST_TAG)).click()
    }

    fun clickDisableRecentsButton() {
        uiDevice.findObjectWithTimeout(By.text("Disable recents")).click()
    }

    fun clickEnableRecentsButton() {
        uiDevice.findObjectWithTimeout(By.text("Enable recents")).click()
    }

    fun assertDaitaChipVisible() {
        uiDevice.findObjectWithTimeout(By.text("Setting: DAITA"))
    }

    fun clickEntryHopSelector() {
        val entry = uiDevice.findObjectWithTimeout(By.res(HOP_SELECTOR_ENTRY_TEST_TAG))

        entry.click()
    }

    fun clickSearchLocation() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SEARCH_BUTTON_TEST_TAG)).click()
    }
}
