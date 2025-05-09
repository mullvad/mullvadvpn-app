package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.waitForStableInActiveWindow
import net.mullvad.mullvadvpn.test.common.extension.expectObjectToDisappearWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class TooManyDevicesPage internal constructor() : Page() {
    private val tooManyDevicesSelector = By.text("Too many devices")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(tooManyDevicesSelector)

        // Assert that we have too many devices
        // And that the continue with login button is disabled
        uiDevice.findObjectWithTimeout(
            By.text("Continue with login").hasParent(By.enabled((false)))
        )
    }

    fun clickRemoveDevice(deviceName: String) {
        val deviceRow = uiDevice.findObjectWithTimeout(By.text(deviceName)).parent
        deviceRow.findObjectWithTimeout(By.desc("Remove")).click()
    }

    fun assertReadyToLogin() {
        uiDevice.findObjectWithTimeout(By.text("Super!"))
    }

    fun clickContinueWithLogin() {
        uiDevice.findObjectWithTimeout(By.text("Continue with login")).click()
    }
}

/** Remove a device and click confirm on the confirmation dialog. */
fun TooManyDevicesPage.removeDeviceFlow(deviceName: String) {
    clickRemoveDevice(deviceName)

    // Wait for the confirmation dialog to appear
    uiDevice.waitForStableInActiveWindow()
    // Confirm logout
    uiDevice.findObjectWithTimeout(By.text("Yes, log out device")).click()

    // Await the device to be removed
    uiDevice.expectObjectToDisappearWithTimeout(By.text(deviceName))
}
