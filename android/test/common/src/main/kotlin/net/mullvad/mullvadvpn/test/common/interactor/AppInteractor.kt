package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.PrivacyPage
import net.mullvad.mullvadvpn.test.common.page.on

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val customApiEndpointConfiguration: ApiEndpointOverride? = null,
) {
    fun launch() {
        device.pressHome()
        // Wait for launcher
        device.wait(Until.hasObject(By.pkg(device.launcherPackageName).depth(0)), LONG_TIMEOUT)

        val targetPackageName = targetContext.packageName
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
        on<PrivacyPage> { clickAgreeOnPrivacyDisclaimer() }
        clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        on<LoginPage>()
    }

    fun launchAndLogIn(accountNumber: String) {
        launchAndEnsureOnLoginPage()
        on<LoginPage> {
            enterAccountNumber(accountNumber)
            clickLoginButton()
        }
    }

    fun clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove(
        timeout: Long = DEFAULT_TIMEOUT
    ) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            // Skipping as notification permissions are not shown.
            return
        }

        val selector = By.text("Allow")

        device.wait(Until.hasObject(selector), timeout)

        try {
            device.findObjectWithTimeout(selector).click()
        } catch (e: IllegalArgumentException) {
            Logger.e("Failed to allow notification permission within timeout ($timeout ms)", e)
        }
    }
}
