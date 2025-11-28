package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.VPN_SETTINGS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SettingsPage internal constructor() : Page() {
    private val settingsSelector = By.text("Settings")
    private val faqAndGuidesSelector = By.text("FAQs & Guides")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(settingsSelector)
    }

    fun clickVpnSettings() {
        uiDevice.findObjectWithTimeout(By.res(VPN_SETTINGS_CELL_TEST_TAG)).click()
    }

    fun clickFaqAndGuides() {
        uiDevice.findObjectWithTimeout(faqAndGuidesSelector).click()
    }

    fun clickDaita() {
        uiDevice.findObjectWithTimeout(By.res(DAITA_CELL_TEST_TAG)).click()
    }

    fun clickMultihop() {
        uiDevice.findObjectWithTimeout(By.res(MULTIHOP_CELL_TEST_TAG)).click()
    }
}
