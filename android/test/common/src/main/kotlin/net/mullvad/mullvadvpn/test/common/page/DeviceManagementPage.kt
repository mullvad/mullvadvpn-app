package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.MANAGE_DEVICES_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.expectObjectToDisappearWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class DeviceManagementPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(MANAGE_DEVICES_SCREEN_TEST_TAG))
    }

    fun removeDevice(deviceName: String) {
        val secondDevice = uiDevice.findObjectWithTimeout(By.text(deviceName)).parent
        secondDevice.findObject(By.clickable(true)).click()
    }

    fun expectDeviceToBeRemoved(deviceName: String) {
        uiDevice.expectObjectToDisappearWithTimeout(By.text(deviceName))
    }
}
