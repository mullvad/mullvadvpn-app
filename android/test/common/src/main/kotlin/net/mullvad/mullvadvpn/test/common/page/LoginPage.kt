package net.mullvad.mullvadvpn.test.common.page

import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
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
        val accountTextField = uiDevice.findObjectWithTimeout(By.clazz("android.widget.EditText"))
        val loginButton = accountTextField.parent.findObject(By.clazz(Button::class.java))
        loginButton.wait(Until.enabled(true), DEFAULT_TIMEOUT)
        loginButton.click()
    }

    fun clickCreateAccount() {
        uiDevice.findObjectWithTimeout(By.text("Create new account")).click()
    }

    fun verifyShowingInvalidAccount() {
        uiDevice.findObjectWithTimeout(invalidAccountNumberSelector, EXTREMELY_LONG_TIMEOUT)
    }

    fun assertHasAccountHistory(accountNumber: String) {
        // This can be improved, if we've entered the same account number in the TextField we might
        // get a false positive.
        uiDevice.findObjectWithTimeout(By.text(accountNumber))
    }

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(loginSelector)
    }
}
