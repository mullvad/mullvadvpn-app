package net.mullvad.mullvadvpn.test.e2e

import androidx.test.runner.AndroidJUnit4
import androidx.test.uiautomator.By
import junit.framework.Assert.assertNotNull
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_FAILURE_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LoginTest : EndToEndTest() {

    @Rule
    @JvmField
    val cleanupAccountTestRule = CleanupAccountTestRule()

    @Test
    fun testLoginWithInvalidCredentials() {
        // Given
        val invalidDummyAccountToken = invalidTestAccountToken

        // When
        app.launch()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
        app.waitForLoginPrompt()
        app.attemptLogin(invalidDummyAccountToken)

        // Then
        device.findObjectWithTimeout(By.text("Login failed"), LOGIN_FAILURE_TIMEOUT)
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

    @Test
    fun testLogout() {
        // Given
        app.launchAndEnsureLoggedIn(validTestAccountToken)

        // When
        app.clickSettingsCog()
        app.clickListItemByText("Account")
        app.clickActionButtonByText("Log out")

        // Then
        assertNotNull(device.findObjectWithTimeout(By.text("Login")))
    }
}
