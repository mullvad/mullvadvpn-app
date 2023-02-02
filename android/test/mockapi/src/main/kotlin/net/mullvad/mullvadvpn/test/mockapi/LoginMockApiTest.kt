package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
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
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert
        device.findObjectWithTimeout(By.text("Login failed"))
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
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
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
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert
        device.findObjectWithTimeout(By.text("Out of time"))
    }
}
