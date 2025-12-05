package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SelectPortPage internal constructor() : Page() {
    private val settingsSelector = By.text("Port")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(settingsSelector)
    }

    fun clickPresetPort(port: Int) {
        uiDevice.clickObjectAwaitIsChecked(By.res(SELECT_PORT_ITEM_X_TEST_TAG.format(port)))
    }
}
