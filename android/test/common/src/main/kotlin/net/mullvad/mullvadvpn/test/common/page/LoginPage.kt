package net.mullvad.mullvadvpn.test.common.page

import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class LoginPage internal constructor() : Page() {
    fun enterAccountNumber(accountNumber: String) {
        uiDevice.findObjectWithTimeout(By.clazz("android.widget.EditText")).text = accountNumber
    }

    fun tapLoginButton() {
        val accountTextField = uiDevice.findObjectWithTimeout(By.clazz("android.widget.EditText"))
        val loginButton = accountTextField.parent.findObject(By.clazz(Button::class.java))
        loginButton.wait(Until.enabled(true), DEFAULT_TIMEOUT)
        loginButton.click()
    }

    fun verifyShowingInvalidAccount() {
        uiDevice.findObjectWithTimeout(By.text("Invalid account number"), EXTREMELY_LONG_TIMEOUT)
    }

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.text("Login"))
    }
}
