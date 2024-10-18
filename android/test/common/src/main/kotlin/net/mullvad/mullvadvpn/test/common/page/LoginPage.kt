package net.mullvad.mullvadvpn.test.common.page

import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class LoginPage(device: UiDevice) : Page(device, By.text("Login")) {
    fun enterAccountNumber(accountNumber: String): LoginPage {
        device.findObjectWithTimeout(By.clazz("android.widget.EditText")).text = accountNumber
        return this
    }

    fun tapLoginButton(): LoginPage {
        val accountTextField = device.findObjectWithTimeout(By.clazz("android.widget.EditText"))
        val loginButton = accountTextField.parent.findObject(By.clazz(Button::class.java))
        loginButton.wait(Until.enabled(true), DEFAULT_TIMEOUT)
        loginButton.click()
        return this
    }

    fun verifyShowingInvalidAccount(): LoginPage {
        device.findObjectWithTimeout(By.text("Invalid account number"), EXTREMELY_LONG_TIMEOUT)
        return this
    }
}
