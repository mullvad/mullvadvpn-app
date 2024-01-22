package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.compose.test.LOGIN_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test

class AccountHistoryMockApiTest : MockApiTest() {

    @Test
    fun testShowAccountHistory() {
        // Arrange
        val validAccountToken = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountToken = validAccountToken
            accountExpiry = currentUtcTimeWithOffsetZero().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"))
        app.clickAccountCog()
        app.clickActionButtonByText("Log out")
        device.findObjectWithTimeout(By.res(LOGIN_INPUT_TEST_TAG)).click()

        // Assert
        assertNotNull(device.findObjectWithTimeout(By.text(validAccountToken.groupWithSpaces())))

        // Try to login with the same account again
        device.findObjectWithTimeout(By.text(validAccountToken.groupWithSpaces())).click()
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"))
    }
}
