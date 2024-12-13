package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage.Companion.SWITCH_TEST_TAG

class DaitaSettingsPage internal constructor() : Page() {
    private val enableSelector = By.text("Enable")
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(DAITA_SCREEN_TEST_TAG))
    }

    fun clickEnableSwitch() {
        val localNetworkSharingCell =
            uiDevice.findObjectWithTimeout(enableSelector).parent
        val localNetworkSharingSwitch =
            localNetworkSharingCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        localNetworkSharingSwitch.click()
    }

    companion object {
        const val DAITA_SCREEN_TEST_TAG = "daita_screen_test_tag"
    }
}
