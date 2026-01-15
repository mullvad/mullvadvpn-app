package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.AccountPage
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.DeviceManagementPage
import net.mullvad.mullvadvpn.test.common.page.on
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
        apiRouter.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = ALMOST_FULL_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_1 to DUMMY_DEVICE_NAME_1
        }

        // Act - go to devices screen
        app.launchAndLogIn(validAccountNumber)

        on<ConnectPage> { clickAccount() }

        on<AccountPage> { clickManageDevices() }

        on<DeviceManagementPage> {
            // Assert - current device is shown but not clickable
            val current = uiDevice.findObjectWithTimeout(By.text("Current device")).parent
            assertNull(current.findObject(By.clickable(true)))

            removeDevice("Yellow Hat")

            // Confirm the removal of the device
            device.findObjectWithTimeout(By.text("Remove")).click()

            expectDeviceToBeRemoved("Yellow Hat")
        }
    }
}
