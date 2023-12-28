package net.mullvad.mullvadvpn.test.e2e

import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.By
import junit.framework.Assert.assertNotNull
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LogoutTest : EndToEndTest() {

    @Rule @JvmField val cleanupAccountTestRule = CleanupAccountTestRule()

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
