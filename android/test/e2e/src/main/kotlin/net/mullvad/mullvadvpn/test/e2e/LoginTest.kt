package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_FAILURE_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LoginTest : EndToEndTest() {

    @RegisterExtension @JvmField val cleanupAccountTestRule = CleanupAccountTestRule()

    @Test
    fun testLoginWithInvalidCredentials() {
        // Given
        val invalidDummyAccountToken = invalidTestAccountToken

        // When
        app.launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(invalidDummyAccountToken)

        // Then
        device.findObjectWithTimeout(By.text("Invalid account number"), LOGIN_FAILURE_TIMEOUT)
    }

    @Test
    fun testLoginWithValidCredentials() {
        // Given
        val token = validTestAccountToken

        // When
        app.launchAndEnsureLoggedIn(token)

        // Then
        app.ensureLoggedIn()
    }
}
