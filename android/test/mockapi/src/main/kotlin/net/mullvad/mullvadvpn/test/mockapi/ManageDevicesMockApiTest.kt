package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.ALMOST_FULL_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_1
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_1
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNull

class ManageDevicesMockApiTest : MockApiTest() {
    @Test
    fun testManageDevicesRemoveDevice() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = ALMOST_FULL_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_1 to DUMMY_DEVICE_NAME_1
        }

        // Act - go to devices screen
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountNumber)
        device.waitForIdle()
        device.dismissChangelogDialogIfShown()
        app.ensureLoggedIn()
        app.clickAccountCog()
        device.findObject(By.res("manage_devices_button_test_tag")).click()

        // Assert - current device is shown but not clickable
        val current = device.findObjectWithTimeout(By.text("Current device")).parent
        assertNull(current.findObject(By.clickable(true)))

        // Act - remove the second device in the list
        val secondDevice = device.findObjectWithTimeout(By.text("Yellow Hat")).parent
        secondDevice.findObject(By.clickable(true)).click()
        app.clickActionButtonByText("Remove")

        // Assert - second device is no longer shown
        assertNull(device.findObject(By.text("Yellow Hat")))
    }
}
