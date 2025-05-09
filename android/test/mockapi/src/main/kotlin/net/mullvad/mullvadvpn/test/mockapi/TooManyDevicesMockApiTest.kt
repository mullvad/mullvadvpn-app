package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.TooManyDevicesPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.page.removeDeviceFlow
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

        on<LoginPage> {
            enterAccountNumber(validAccountNumber)
            tapLoginButton()
        }

        // Assert that we have too many devices
        on<TooManyDevicesPage> {
            removeDeviceFlow(FULL_DEVICE_LIST.values.first())

            assertReadyToLogin()
            clickContinueWithLogin()
        }

        // Assert that we are logged in
        device.dismissChangelogDialogIfShown()
        app.ensureLoggedIn()
    }
}
