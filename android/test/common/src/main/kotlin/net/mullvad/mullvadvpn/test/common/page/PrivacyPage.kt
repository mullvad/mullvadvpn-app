package net.mullvad.mullvadvpn.test.common.page

import android.os.Build
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class PrivacyPage internal constructor() : Page() {
    private val privacySelector = By.text("Privacy")
    private val agreeSelector = By.text("Agree and continue")
    private val allowSelector = By.text("Allow")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(privacySelector)
    }

    fun clickAgreeOnPrivacyDisclaimer() {
        uiDevice.findObjectWithTimeout(agreeSelector).click()
    }

    fun clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove(
        timeout: Long = DEFAULT_TIMEOUT
    ) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            // Skipping as notification permissions are not shown.
            return
        }

        uiDevice.wait(Until.hasObject(allowSelector), timeout)

        try {
            uiDevice.findObjectWithTimeout(allowSelector).click()
        } catch (e: IllegalArgumentException) {
            throw IllegalArgumentException(
                "Failed to allow notification permission within timeout ($timeout)"
            )
        }
    }
}
