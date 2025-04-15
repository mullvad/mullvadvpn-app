package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_6
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_6
import net.mullvad.mullvadvpn.test.mockapi.constant.FULL_DEVICE_LIST
import org.junit.jupiter.api.Test

class TooManyDevicesMockApiTest : MockApiTest() {
    @Test
    fun testRemoveDeviceSuccessfulAndLogin() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = FULL_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_6 to DUMMY_DEVICE_NAME_6
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountNumber)

        // Assert that we have too many devices
        device.findObjectWithTimeout(By.text("Too many devices"))
        // And that the continue with login button is disabled
        device.findObjectWithTimeout(By.text("Continue with login").hasParent(By.enabled((false))))

        // Act
        // Wait until the application is idle to avoid skipping input events that are filtered out
        // depending on lifecycle state (dropUnlessResumed).
        device.waitForIdle()
        app.attemptToRemoveDevice()

        // Assert that a device was removed
        device.findObjectWithTimeout(By.text("Super!"))

        // Act
        app.clickActionButtonByText("Continue with login")

        // Assert that we are logged in
        device.dismissChangelogDialogIfShown()
        app.ensureLoggedIn()
    }
}
