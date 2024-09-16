package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
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
        val validTestAccountNumber = accountTestRule.validAccountNumber

        // When
        app.launchAndEnsureLoggedIn(validTestAccountNumber)

        // Then
        app.ensureLoggedIn()
    }

    @Test
    @Disabled("Failed login attempts are highly rate limited and cause test flakiness")
    fun testLoginWithInvalidCredentials() {
        // Given
        val invalidDummyAccountNumber = accountTestRule.invalidAccountNumber

        // When
        app.launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(invalidDummyAccountNumber)

        // Then
        device.findObjectWithTimeout(By.text("Invalid account number"), EXTREMELY_LONG_TIMEOUT)
    }
}
