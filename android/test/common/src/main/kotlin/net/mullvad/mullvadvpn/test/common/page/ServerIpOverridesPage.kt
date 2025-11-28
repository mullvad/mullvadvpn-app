package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_IMPORT_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class ServerIpOverridesPage internal constructor() : Page() {
    private val serverIpOverrideSelector = By.text("Server IP override")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(serverIpOverrideSelector)
    }

    fun clickImportButton() {
        uiDevice.findObjectWithTimeout(By.res(SERVER_IP_OVERRIDE_IMPORT_TEST_TAG)).click()
    }
}
