package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.UiObjectNotFoundException
import androidx.test.uiautomator.Until
import androidx.test.uiautomator.waitForStableInActiveWindow
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.HOP_SELECTOR_ENTRY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_MENU_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.misc.TestRelay

class SelectLocationPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_SCREEN_TEST_TAG))
    }

    fun clickLocationExpandButton(locationName: String) {
        val locationCell =
            uiDevice
                .findObjectWithTimeout(
                    By.textContains(locationName).hasAncestor(By.res(GEOLOCATION_ITEM_TAG))
                )
                .parent
                .parent
        val expandButton = locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
        expandButton.click()
    }

    fun clickLocationCell(locationName: String) {
        uiDevice.findObjectWithTimeout(By.text(locationName)).click()
    }

    fun scrollUntilText(text: String, direction: Direction = Direction.DOWN) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_LIST_TEST_TAG))
        scrollView2.scrollUntil(direction, Until.hasObject(By.text(text)))
    }

    fun scrollPercentage(direction: Direction, percentage: Float) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_LIST_TEST_TAG))
        scrollView2.scroll(direction, percentage)
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

    fun expandAndClickRelay(testRelay: TestRelay) {
        clickLocationExpandButton(testRelay.country)
        uiDevice.waitForStableInActiveWindow()
        scrollUntilText(testRelay.city, Direction.DOWN)
        // Due to the fab obstructing the expand button we can sometimes scroll until the city is
        // visible but the expand button for the city is not. In that case we will scroll a bit more
        // and try again.
        try {
            clickLocationExpandButton(testRelay.city)
        } catch (_: UiObjectNotFoundException) {
            scrollPercentage(Direction.DOWN, 0.1f)
            clickLocationExpandButton(testRelay.city)
        }
        uiDevice.waitForStableInActiveWindow()
        scrollUntilText(testRelay.relay, Direction.DOWN)
        clickLocationCell(testRelay.relay)
    }

    fun assertDaitaChipVisible() {
        uiDevice.findObjectWithTimeout(By.text("Setting: DAITA"))
    }

    fun clickEntryHopSelector() {
        val entry = uiDevice.findObjectWithTimeout(By.res(HOP_SELECTOR_ENTRY_TEST_TAG))

        entry.click()
    }
}
