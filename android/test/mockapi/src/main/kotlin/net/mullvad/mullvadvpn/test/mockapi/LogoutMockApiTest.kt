package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test

class LogoutMockApiTest : MockApiTest() {

    @Test
    fun testLoginWithValidCredentialsToUnexpiredAccountAndLogout() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountNumber)
        device.dismissChangelogDialogIfShown()
        app.ensureLoggedIn()
        app.clickAccountCog()
        app.clickActionButtonByText("Log out")

        // Assert
        assertNotNull(device.findObjectWithTimeout(By.text("Login")))
    }
}
