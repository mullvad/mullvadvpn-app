package net.mullvad.mullvadvpn.test.mockapi


import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.compose.test.LOGIN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_INTERACTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class LoginMockApiTest : MockApiTest() {
    @Test
    fun testLoginWithInvalidCredentials() {
        // Arrange
        val validAccountToken = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountToken = null
            accountExpiry = currentUtcTimeWithOffsetZero().plusDays(1)
        }
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert
        val result =
            device
                .findObject(By.res(LOGIN_TITLE_TEST_TAG))
                .wait(Until.textEquals("Login failed"), DEFAULT_INTERACTION_TIMEOUT)

        assertTrue(result)
    }

    @Test
    fun testLoginWithValidCredentialsToUnexpiredAccount() {
        // Arrange
        val validAccountToken = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountToken = validAccountToken
            accountExpiry = currentUtcTimeWithOffsetZero().plusDays(1)
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"))
    }

    @Test
    fun testLoginWithValidCredentialsToExpiredAccount() {
        // Arrange
        val validAccountToken = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountToken = validAccountToken
            accountExpiry = currentUtcTimeWithOffsetZero().minusDays(1)
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert
        device.findObjectWithTimeout(By.text("Out of time"))
    }
}
