package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_BUTTON_TEST_TAG
<<<<<<< Updated upstream
||||||| Stash base
=======
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_REVEAL_INPUT_BUTTON_TEST_TAG
>>>>>>> Stashed changes
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_SCREEN_DELETE_ACCOUNT_HISTORY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_SETTINGS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class LoginPage internal constructor() : Page() {
    private val invalidAccountNumberSelector = By.text("Invalid account number")
    private val loginSelector = By.text("Log in")

    fun clickSettings() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON_TEST_TAG)).click()
    }

    fun enterAccountNumber(accountNumber: String) {
        uiDevice.findObjectWithTimeout(By.clazz("android.widget.EditText")).text = accountNumber
    }

    fun clickLoginButton() {
        val loginButton = uiDevice.findObjectWithTimeout(By.res(LOGIN_BUTTON_TEST_TAG))
        loginButton.wait(Until.enabled(true), DEFAULT_TIMEOUT)
        loginButton.click()
    }

    fun clickCreateAccount() {
        uiDevice.findObjectWithTimeout(By.text("Create new account")).click()
    }

    fun verifyShowingInvalidAccount() {
        uiDevice.findObjectWithTimeout(invalidAccountNumberSelector, EXTREMELY_LONG_TIMEOUT)
    }

    fun toggleRevealInput() {
        uiDevice.findObjectWithTimeout(By.res(LOGIN_REVEAL_INPUT_BUTTON_TEST_TAG)).click()
    }

    fun assertHasAccountHistory(accountNumber: String) {
        // This can be improved, if we've entered the same account number in the TextField we might
        // get a false positive.
        uiDevice.findObjectWithTimeout(By.text(accountNumber))
    }

    fun deleteAccountHistory() {
        uiDevice.findObjectWithTimeout(By.res(LOGIN_SCREEN_DELETE_ACCOUNT_HISTORY_TEST_TAG)).click()
    }

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(loginSelector)
    }
}
