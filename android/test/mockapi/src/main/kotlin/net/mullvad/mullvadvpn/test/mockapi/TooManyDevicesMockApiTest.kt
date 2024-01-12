package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_1
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_3
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_4
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_5
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_1
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_3
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_4
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_5
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import org.junit.jupiter.api.Test

class TooManyDevicesMockApiTest : MockApiTest() {
    @Test
    fun testRemoveDeviceSuccessfulAndLogin() {
        // Arrange
        val validAccountToken = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountToken = validAccountToken
            accountExpiry = currentUtcTimeWithOffsetZero().plusMonths(1)
            devices =
                mutableMapOf(
                    DUMMY_ID_1 to DUMMY_DEVICE_NAME_1,
                    DUMMY_ID_2 to DUMMY_DEVICE_NAME_2,
                    DUMMY_ID_3 to DUMMY_DEVICE_NAME_3,
                    DUMMY_ID_4 to DUMMY_DEVICE_NAME_4,
                    DUMMY_ID_5 to DUMMY_DEVICE_NAME_5
                )
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert that we have too many devices
        device.findObjectWithTimeout(By.text("Too many devices"))
        // And that the continue with login button is disabled
        device.findObjectWithTimeout(By.text("Continue with login")).isEnabled

        // Act
        app.attemptToRemoveDevice()

        // Assert that a device was removed
        device.findObjectWithTimeout(By.text("Super!"))

        // Act
        app.clickActionButtonByText("Continue with login")

        // Assert that we are logged in
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"))
    }
}
