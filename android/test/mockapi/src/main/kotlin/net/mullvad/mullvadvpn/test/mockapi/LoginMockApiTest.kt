package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.By
import java.time.OffsetDateTime
import java.time.temporal.ChronoUnit
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
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
            accountExpiry =
                OffsetDateTime.now().plusDays(1).truncatedTo(ChronoUnit.SECONDS)
        }
        app.launch(endpoint)

        // Act
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
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
            accountExpiry =
                OffsetDateTime.now().plusDays(1).truncatedTo(ChronoUnit.SECONDS)
        }

        // Act
        app.launch(endpoint)
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
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
            accountExpiry =
                OffsetDateTime.now().minusDays(1).truncatedTo(ChronoUnit.SECONDS)
        }

        // Act
        app.launch(endpoint)
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
        app.attemptLogin(validAccountToken)

        // Assert
        device.findObjectWithTimeout(By.text("Out of time"))
    }
}
