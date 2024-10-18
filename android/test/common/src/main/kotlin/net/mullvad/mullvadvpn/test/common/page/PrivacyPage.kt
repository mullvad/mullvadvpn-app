package net.mullvad.mullvadvpn.test.common.page

import android.os.Build
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class PrivacyPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
       uiDevice.findObjectWithTimeout(By.text("Privacy"))
    }

    fun clickAgreeOnPrivacyDisclaimer() {
        uiDevice.findObjectWithTimeout(By.text("Agree and continue")).click()
    }

    fun clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove(
        timeout: Long = DEFAULT_TIMEOUT
    ) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            // Skipping as notification permissions are not shown.
            return
        }

        val selector = By.text("Allow")

        uiDevice.wait(Until.hasObject(selector), timeout)

        try {
            uiDevice.findObjectWithTimeout(selector).click()
        } catch (e: IllegalArgumentException) {
            throw IllegalArgumentException(
                "Failed to allow notification permission within timeout ($timeout)"
            )
        }
    }
}
