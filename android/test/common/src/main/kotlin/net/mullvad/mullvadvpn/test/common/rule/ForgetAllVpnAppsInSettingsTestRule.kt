package net.mullvad.mullvadvpn.test.common.rule

import android.content.Intent
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.common.extension.findObjectByCaseInsensitiveText
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import org.junit.jupiter.api.extension.BeforeTestExecutionCallback
import org.junit.jupiter.api.extension.ExtensionContext

class ForgetAllVpnAppsInSettingsTestRule : BeforeTestExecutionCallback {
    override fun beforeTestExecution(context: ExtensionContext) {
        val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        targetContext.startActivity(
            Intent(Settings.ACTION_VPN_SETTINGS).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            }
        )
        val vpnSettingsButtons =
            device.findObjects(By.res(SETTINGS_PACKAGE, VPN_SETTINGS_BUTTON_ID))
        vpnSettingsButtons.forEach { button ->
            button.click()

            try {
                device.findObjectWithTimeout(By.text(FORGET_VPN_VPN_BUTTON_TEXT)).click()
                device.findObjectByCaseInsensitiveText(FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT).click()
            } catch (_: Exception) {
                device.findObjectWithTimeout(By.text(DELETE_VPN_PROFILE_TEXT)).click()
                device.findObjectWithTimeout(By.text(DELETE_VPN_CONFIRM_BUTTON_TEXT_REGEXP)).click()
            }
        }
    }

    companion object {
        private const val FORGET_VPN_VPN_BUTTON_TEXT = "Forget VPN"
        private const val DELETE_VPN_PROFILE_TEXT = "Delete VPN profile"
        private const val FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT = "Forget"
        // Samsung S22 shows "Delete"
        // Stock Android shows "DELETE"
        private val DELETE_VPN_CONFIRM_BUTTON_TEXT_REGEXP = Pattern.compile("DELETE|Delete")
        private const val SETTINGS_PACKAGE = "com.android.settings"
        private const val VPN_SETTINGS_BUTTON_ID = "settings_button"
    }
}
