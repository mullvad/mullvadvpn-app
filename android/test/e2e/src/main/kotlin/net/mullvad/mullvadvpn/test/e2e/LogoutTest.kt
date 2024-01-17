package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LogoutTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @Test
    fun testLogout() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        app.clickAccountCog()
        app.clickActionButtonByText("Log out")

        // Then
        assertNotNull(device.findObjectWithTimeout(By.text("Login")))
    }

    @Test
    fun testCreateAccountAndLogout() {
        // Given
        app.launchAndCreateAccount()

        // When
        app.clickAccountCog()
        app.clickActionButtonByText("Log out")

        // Then
        assertNotNull(device.findObjectWithTimeout(By.text("Login")))
    }
}
