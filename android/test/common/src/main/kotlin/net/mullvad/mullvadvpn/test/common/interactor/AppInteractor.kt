package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import android.widget.Button
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.lib.ui.tag.CONNECT_CARD_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_INFO_CONNECTION_IN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_INFO_CONNECTION_OUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.OUT_OF_TIME_SCREEN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_ACCOUNT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_SETTINGS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val targetPackageName: String,
) {
    fun launch(customApiEndpointConfiguration: ApiEndpointOverride? = null) {
        device.pressHome()
        // Wait for launcher
        device.wait(Until.hasObject(By.pkg(device.launcherPackageName).depth(0)), LONG_TIMEOUT)

        val intent =
            targetContext.packageManager.getLaunchIntentForPackage(targetPackageName)?.apply {
                // Clear out any previous instances
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
                if (customApiEndpointConfiguration != null) {
                    putApiEndpointConfigurationExtra(customApiEndpointConfiguration)
                }
            }
        targetContext.startActivity(intent)
        device.wait(Until.hasObject(By.pkg(targetPackageName).depth(0)), LONG_TIMEOUT)
    }

    fun launchAndEnsureOnLoginPage() {
        launch()
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        waitForLoginPrompt()
    }

    fun launchAndEnsureLoggedIn(accountNumber: String) {
        launchAndEnsureOnLoginPage()
        attemptLogin(accountNumber)
        device.dismissChangelogDialogIfShown()
        ensureLoggedIn()
    }

    fun attemptLogin(accountNumber: String) {
        val loginObject =
            device.findObjectWithTimeout(By.clazz("android.widget.EditText")).apply {
                text = accountNumber
            }
        val loginButton = loginObject.parent.findObject(By.clazz(Button::class.java))
        loginButton.wait(Until.enabled(true), DEFAULT_TIMEOUT)
        loginButton.click()
    }

    fun attemptCreateAccount() {
        device.findObjectWithTimeout(By.text("Create account")).click()
    }

    fun ensureAccountCreated(accountNumber: String? = null) {
        device.findObjectWithTimeout(By.text("Congrats!"), VERY_LONG_TIMEOUT)
        accountNumber?.let { device.findObjectWithTimeout(By.text(accountNumber), DEFAULT_TIMEOUT) }
    }

    fun ensureAccountCreationFailed() {
        device.findObjectWithTimeout(By.text("Failed to create account"), EXTREMELY_LONG_TIMEOUT)
    }

    fun ensureLoggedIn() {
        device.findObjectWithTimeout(By.text("DISCONNECTED"), VERY_LONG_TIMEOUT)
    }

    fun ensureOutOfTime() {
        device.findObjectWithTimeout(By.res(OUT_OF_TIME_SCREEN_TITLE_TEST_TAG))
    }

    fun ensureAccountScreen() {
        device.findObjectWithTimeout(By.text("Account"))
    }

    fun extractOutIpv4Address(): String {
        device.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG)).click()
        return device
            .findObjectWithTimeout(
                // Text exist and contains IP address
                By.res(LOCATION_INFO_CONNECTION_OUT_TEST_TAG).textContains("."),
                VERY_LONG_TIMEOUT,
            )
            .text
    }

    fun extractInIpv4Address(): String {
        device.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG)).click()
        val inString =
            device
                .findObjectWithTimeout(
                    By.res(LOCATION_INFO_CONNECTION_IN_TEST_TAG),
                    VERY_LONG_TIMEOUT,
                )
                .text

        val extractedIpAddress = inString.split(" ")[0].split(":")[0]
        return extractedIpAddress
    }

    fun clickSettingsCog() {
        device.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON_TEST_TAG)).click()
    }

    fun clickAccountCog() {
        device.findObjectWithTimeout(By.res(TOP_BAR_ACCOUNT_BUTTON_TEST_TAG)).click()
    }

    fun clickListItemByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun clickActionButtonByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun waitForLoginPrompt(timeout: Long = VERY_LONG_TIMEOUT) {
        device.findObjectWithTimeout(By.text("Login"), timeout)
    }

    fun attemptToRemoveDevice() {
        device.findObjectWithTimeout(By.desc("Remove")).click()
        clickActionButtonByText("Yes, log out device")
    }

    fun dismissStorePasswordPromptIfShown() {
        try {
            device.waitForIdle()
            val selector = By.textContains("password")
            device.wait(Until.hasObject(selector), DEFAULT_TIMEOUT)
            device.pressBack()
        } catch (e: IllegalArgumentException) {
            // This is OK since it means the password prompt wasn't shown.
        }
    }
}
