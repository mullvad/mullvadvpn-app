package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class PrivacyPage internal constructor() : Page() {
    private val privacySelector = By.text("Privacy")
    private val agreeSelector = By.text("Agree and continue")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(privacySelector)
    }

    fun clickAgreeOnPrivacyDisclaimer() {
        uiDevice.findObjectWithTimeout(agreeSelector).click()
    }
}
