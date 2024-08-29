package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.test.common.constant.APP_LAUNCH_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_INTERACTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_FAILURE_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_PROMPT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val targetPackageName: String
) {
    fun launch(customApiEndpointConfiguration: ApiEndpoint.Custom? = null) {
        device.pressHome()
        // Wait for launcher
        device.wait(
            Until.hasObject(By.pkg(device.launcherPackageName).depth(0)),
            APP_LAUNCH_TIMEOUT
        )

        val intent =
            targetContext.packageManager.getLaunchIntentForPackage(targetPackageName)?.apply {
                // Clear out any previous instances
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
                if (customApiEndpointConfiguration != null) {
                    putApiEndpointConfigurationExtra(customApiEndpointConfiguration)
                }
            }
        targetContext.startActivity(intent)
        device.wait(Until.hasObject(By.pkg(targetPackageName).depth(0)), APP_LAUNCH_TIMEOUT)
    }

    fun launchAndEnsureLoggedIn(accountNumber: String) {
        launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        waitForLoginPrompt()
        attemptLogin(accountNumber)
        ensureLoggedIn()
    }

    fun attemptLogin(accountNumber: String) {
        val loginObject =
            device.findObjectWithTimeout(By.clazz("android.widget.EditText")).apply {
                text = accountNumber
            }
        val loginButton = loginObject.parent.findObject(By.clazz(Button::class.java))
        loginButton.wait(Until.enabled(true), DEFAULT_INTERACTION_TIMEOUT)
        loginButton.click()
    }

    fun attemptCreateAccount() {
        device.findObjectWithTimeout(By.text("Create account")).click()
    }

    fun ensureAccountCreated(accountNumber: String? = null) {
        device.findObjectWithTimeout(By.text("Congrats!"), LOGIN_TIMEOUT)
        accountNumber?.let {
            device.findObjectWithTimeout(By.text(accountNumber), DEFAULT_INTERACTION_TIMEOUT)
        }
    }

    fun ensureAccountCreationFailed() {
        device.findObjectWithTimeout(By.text("Failed to create account"), LOGIN_FAILURE_TIMEOUT)
    }

    fun ensureLoggedIn() {
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"), LOGIN_TIMEOUT)
    }

    fun ensureOutOfTime() {
        device.findObjectWithTimeout(By.res("out_of_time_screen_title_test_tag"))
    }

    fun ensureAccountScreen() {
        device.findObjectWithTimeout(By.text("Account"))
    }

    fun extractIpAddress(): String {
        device.findObjectWithTimeout(By.res("location_info_test_tag")).click()
        return device
            .findObjectWithTimeout(
                // Text exist and contains IP address
                By.res("location_info_connection_out_test_tag").textContains("."),
                CONNECTION_TIMEOUT
            )
            .text
            .extractIpAddress()
    }

    fun clickSettingsCog() {
        device.findObjectWithTimeout(By.res("top_bar_settings_button")).click()
    }

    fun clickAccountCog() {
        device.findObjectWithTimeout(By.res("top_bar_account_button")).click()
    }

    fun clickListItemByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun clickActionButtonByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun waitForLoginPrompt(timeout: Long = LOGIN_PROMPT_TIMEOUT) {
        device.findObjectWithTimeout(By.text("Login"), timeout)
    }

    fun attemptToRemoveDevice() {
        device.findObjectWithTimeout(By.desc("Remove")).click()
        clickActionButtonByText("Yes, log out device")
    }

    private fun String.extractIpAddress(): String {
        return split(" ")[1].split(" ")[0]
    }
}
