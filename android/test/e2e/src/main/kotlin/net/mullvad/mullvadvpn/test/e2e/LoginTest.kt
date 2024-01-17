package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_FAILURE_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LoginTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @Test
    fun testLoginWithValidCredentials() {
        // Given
        val validTestAccountToken = accountTestRule.validAccountNumber

        // When
        app.launchAndEnsureLoggedIn(validTestAccountToken)

        // Then
        app.ensureLoggedIn()
    }

    @Test
    @Disabled("Disabled to avoid getting rate-limited.")
    fun testLoginWithInvalidCredentials() {
        // Given
        val invalidDummyAccountToken = accountTestRule.invalidAccountNumber

        // When
        app.launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(invalidDummyAccountToken)

        // Then
        device.findObjectWithTimeout(By.text("Invalid account number"), LOGIN_FAILURE_TIMEOUT)
    }
}
