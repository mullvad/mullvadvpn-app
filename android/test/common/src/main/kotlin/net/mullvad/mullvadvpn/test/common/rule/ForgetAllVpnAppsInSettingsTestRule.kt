package net.mullvad.mullvadvpn.test.common.rule

import android.content.Intent
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.common.extension.findObjectByCaseInsensitiveText
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import org.junit.rules.TestWatcher
import org.junit.runner.Description

class ForgetAllVpnAppsInSettingsTestRule : TestWatcher() {
    override fun starting(description: Description) {
        val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        val targetContext = InstrumentationRegistry.getInstrumentation().targetContext
        targetContext.startActivity(
            Intent(Settings.ACTION_VPN_SETTINGS).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
            }
        )
        val vpnSettingsButtons =
            device.findObjects(By.res(SETTINGS_PACKAGE, VPN_SETTINGS_BUTTON_ID))
        vpnSettingsButtons?.forEach { button ->
            button.click()
            device.findObjectWithTimeout(By.text(FORGET_VPN_VPN_BUTTON_TEXT)).click()
            device.findObjectByCaseInsensitiveText(FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT).click()
        }
    }

    companion object {
        private const val FORGET_VPN_VPN_BUTTON_TEXT = "Forget VPN"
        private const val FORGET_VPN_VPN_CONFIRM_BUTTON_TEXT = "Forget"
        private const val SETTINGS_PACKAGE = "com.android.settings"
        private const val VPN_SETTINGS_BUTTON_ID = "settings_button"
    }
}
