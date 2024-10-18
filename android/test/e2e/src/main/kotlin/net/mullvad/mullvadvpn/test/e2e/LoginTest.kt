package net.mullvad.mullvadvpn.test.e2e

import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LoginTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @Test
    fun testLoginWithValidCredentials() {
        val validTestAccountNumber = accountTestRule.validAccountNumber

        app.launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()

        LoginPage(device)
            .enterAccountNumber(validTestAccountNumber)
            .tapLoginButton()

        device.dismissChangelogDialogIfShown()
        ConnectPage(device)
    }

    @Test
    @Disabled("Failed login attempts are highly rate limited and cause test flakiness")
    fun testLoginWithInvalidCredentials() {
        val invalidDummyAccountNumber = accountTestRule.invalidAccountNumber

        app.launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()

        LoginPage(device)
            .enterAccountNumber(invalidDummyAccountNumber)
            .tapLoginButton()
            .verifyShowingInvalidAccount()
    }
}
