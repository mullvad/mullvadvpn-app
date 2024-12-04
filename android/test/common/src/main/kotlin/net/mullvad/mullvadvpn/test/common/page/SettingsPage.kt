package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class SettingsPage internal constructor() : Page() {
    private val bySettingsText = By.text("Settings")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(bySettingsText)
    }

    fun clickVpnSettings() {
        uiDevice.findObjectWithTimeout(By.res(VPN_SETTINGS_CELL_TEST_TAG)).click()
    }

    companion object {
        const val VPN_SETTINGS_CELL_TEST_TAG = "vpn_settings_cell_test_tag"
    }
}
