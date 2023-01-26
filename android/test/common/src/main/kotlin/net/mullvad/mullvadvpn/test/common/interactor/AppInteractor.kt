package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import android.widget.ImageButton
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.endpoint.CustomApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.test.common.constant.APP_LAUNCH_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_PROMPT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LOGIN_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.MULLVAD_PACKAGE
import net.mullvad.mullvadvpn.test.common.constant.SETTINGS_COG_ID
import net.mullvad.mullvadvpn.test.common.constant.TUNNEL_INFO_ID
import net.mullvad.mullvadvpn.test.common.constant.TUNNEL_OUT_ADDRESS_ID
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context
) {
    fun launch(customApiEndpointConfiguration: CustomApiEndpointConfiguration? = null) {
        device.pressHome()
        // Wait for launcher
        device.wait(
            Until.hasObject(By.pkg(device.launcherPackageName).depth(0)),
            APP_LAUNCH_TIMEOUT
        )

        val intent =
            targetContext.packageManager.getLaunchIntentForPackage(MULLVAD_PACKAGE)?.apply {
                // Clear out any previous instances
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
                if (customApiEndpointConfiguration != null) {
                    putApiEndpointConfigurationExtra(customApiEndpointConfiguration)
                }
            }
        targetContext.startActivity(intent)
        device.wait(
            Until.hasObject(By.pkg(MULLVAD_PACKAGE).depth(0)),
            APP_LAUNCH_TIMEOUT
        )
    }

    fun launchAndEnsureLoggedIn(accountToken: String) {
        launch()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel31AndAbove()
        attemptLogin(accountToken)
        ensureLoggedIn()
    }

    fun attemptLogin(accountToken: String) {
        val loginObject = device.findObjectWithTimeout(By.clazz("android.widget.EditText"))
            .apply { text = accountToken }
        loginObject.parent.findObject(By.clazz(ImageButton::class.java)).click()
    }

    fun ensureLoggedIn() {
        device.findObjectWithTimeout(By.text("UNSECURED CONNECTION"), LOGIN_TIMEOUT)
    }

    fun extractIpAddress(): String {
        device.findObjectWithTimeout(By.res(TUNNEL_INFO_ID)).click()
        return device.findObjectWithTimeout(
            By.res(TUNNEL_OUT_ADDRESS_ID),
            CONNECTION_TIMEOUT
        ).text.extractIpAddress()
    }

    fun clickSettingsCog() {
        device.findObjectWithTimeout(By.res(SETTINGS_COG_ID)).click()
    }

    fun clickListItemByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun clickActionButtonByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }

    fun waitForLoginPrompt(
        timeout: Long = LOGIN_PROMPT_TIMEOUT
    ) {
        device.findObjectWithTimeout(By.text("Login"), timeout)
    }

    private fun String.extractIpAddress(): String {
        return split("  ")[1].split(" ")[0]
    }
}
