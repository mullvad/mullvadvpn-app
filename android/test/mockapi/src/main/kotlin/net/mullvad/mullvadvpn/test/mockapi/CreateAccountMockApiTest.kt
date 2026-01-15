package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.WelcomePage
import net.mullvad.mullvadvpn.test.common.page.dismissStorePasswordPromptIfShown
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Test

class CreateAccountMockApiTest : MockApiTest() {
    @Test
    fun testCreateAccountSuccessful() {
        // Arrange
        val createdAccountNumber = "1234123412341234"
        apiRouter.apply {
            expectedAccountNumber = createdAccountNumber
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> { clickCreateAccount() }

        device.dismissStorePasswordPromptIfShown()

        on<WelcomePage> {
            // Ensure account number is visible
            device.findObjectWithTimeout(By.text("1234 1234 1234 1234"))
        }
    }

    @Test
    fun testCreateAccountFailed() {
        // Arrange
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> {
            clickCreateAccount()
            device.findObjectWithTimeout(By.text("Failed to create account"))
        }
    }
}
