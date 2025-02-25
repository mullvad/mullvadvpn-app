package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.compose.test.LOGIN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class LoginMockApiTest : MockApiTest() {
    @Test
    fun testLoginWithInvalidCredentials() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = null
            accountExpiry = ZonedDateTime.now().plusDays(1)
        }
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountNumber)
        app.attemptLogin(validAccountNumber)

        // Assert
        val result =
            device
                .findObject(By.res(LOGIN_TITLE_TEST_TAG))
                .wait(Until.textEquals("Login failed"), DEFAULT_TIMEOUT)

        assertTrue(result)
    }

    @Test
    fun testLoginWithValidCredentialsToUnexpiredAccount() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusDays(1)
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

        // Assert
        app.ensureLoggedIn()
    }

    @Test
    fun testLoginWithValidCredentialsToExpiredAccount() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().minusDays(1)
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

        // Assert
        app.ensureOutOfTime()
    }
}
