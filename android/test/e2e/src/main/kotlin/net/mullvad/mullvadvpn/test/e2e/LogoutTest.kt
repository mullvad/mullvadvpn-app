package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LogoutTest : EndToEndTest() {

    @RegisterExtension @JvmField val cleanupAccountTestRule = CleanupAccountTestRule()

    @Test
    fun testLogout() {
        // Given
        app.launchAndEnsureLoggedIn(validTestAccountToken)

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
