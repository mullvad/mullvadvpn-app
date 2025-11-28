package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class ImportOverridesBottomSheet internal constructor() : Page() {
    private val importOverrideSelector = By.text("Import new overrides by")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(importOverrideSelector)
    }

    fun clickText() {
        uiDevice.findObjectWithTimeout(By.res(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG)).click()
    }
}
