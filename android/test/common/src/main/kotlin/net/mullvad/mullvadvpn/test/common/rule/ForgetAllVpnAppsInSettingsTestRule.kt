package net.mullvad.mullvadvpn.test.common.rule

import android.content.Intent
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectByCaseInsensitiveText
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.hasObjectWithTimeout
import org.junit.jupiter.api.extension.BeforeTestExecutionCallback
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.fail

class ForgetAllVpnAppsInSettingsTestRule : BeforeTestExecutionCallback {
    override fun beforeTestExecution(context: ExtensionContext) {
        val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        targetContext.startActivity(
            Intent(Settings.ACTION_VPN_SETTINGS).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            }
        )

        val vpnSettingsSelector = By.res(SETTINGS_PACKAGE, VPN_SETTINGS_BUTTON_ID)
        device.wait(Until.hasObject(vpnSettingsSelector), DEFAULT_TIMEOUT)
        val vpnSettingsButtons = device.findObjects(vpnSettingsSelector)

        vpnSettingsButtons
            .filter { !it.isHardcodedVpn() }
            .forEach { button ->
                button.click()

                if (device.hasObjectWithTimeout(By.text(FORGET_VPN_VPN_BUTTON_TEXT))) {
                    device.findObjectWithTimeout(By.text(FORGET_VPN_VPN_BUTTON_TEXT)).click()
                    device
                        .findObjectByCaseInsensitiveText(FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT)
                        .click()
                } else if (device.hasObjectWithTimeout(By.text(DELETE_VPN_PROFILE_TEXT))) {
                    device.findObjectWithTimeout(By.text(DELETE_VPN_PROFILE_TEXT)).click()
                    device
                        .findObjectWithTimeout(By.text(DELETE_VPN_CONFIRM_BUTTON_TEXT_REGEXP))
                        .click()
                } else if (device.hasObjectWithTimeout(By.text(FORGET_VPN_BUTTON_TEXT))) {
                    device.findObjectWithTimeout(By.text(FORGET_VPN_BUTTON_TEXT)).click()
                } else {
                    fail("Unable to find forget or delete button")
                }
            }
    }

    private fun UiObject2.isHardcodedVpn(): Boolean =
        parent.parent.children.any { uiObject ->
            HARDCODED_VPN_PROFILE_NAMES.any { uiObject.hasObject(By.text(it)) }
        }

    companion object {
        private val HARDCODED_VPN_PROFILE_NAMES = listOf("VPN by Google")

        private const val FORGET_VPN_VPN_BUTTON_TEXT = "Forget VPN"
        private const val FORGET_VPN_BUTTON_TEXT = "Forget" // Android 16, Pixel 8a
        private const val DELETE_VPN_PROFILE_TEXT = "Delete VPN profile"
        private const val FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT = "Forget"
        // Samsung S22 shows "Delete"
        // Stock Android shows "DELETE"
        private val DELETE_VPN_CONFIRM_BUTTON_TEXT_REGEXP = Pattern.compile("DELETE|Delete")
        private const val SETTINGS_PACKAGE = "com.android.settings"
        private const val VPN_SETTINGS_BUTTON_ID = "settings_button"
    }
}
