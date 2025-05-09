package net.mullvad.mullvadvpn.test.common.interactor

import android.content.Context
import android.content.Intent
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.lib.endpoint.putApiEndpointConfigurationExtra
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.on

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val targetPackageName: String,
    private val customApiEndpointConfiguration: ApiEndpointOverride? = null,
) {
    fun launch() {
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
        on<LoginPage>()
    }

    fun launchAndLogIn(accountNumber: String) {
        launchAndEnsureOnLoginPage()
        on<LoginPage> {
            enterAccountNumber(accountNumber)
            clickLoginButton()
        }
    }
}
