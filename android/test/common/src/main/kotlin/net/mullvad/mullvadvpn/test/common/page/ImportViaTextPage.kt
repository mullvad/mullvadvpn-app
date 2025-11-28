package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_IMPORT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_TEXT_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class ImportViaTextPage internal constructor() : Page() {
    private val importViaTextSelector = By.text("Import via text")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(importViaTextSelector)
    }

    fun input(text: String) {
        uiDevice.findObjectWithTimeout(By.res(SERVER_IP_OVERRIDES_TEXT_INPUT_TEST_TAG)).text = text
    }

    fun clickImport() {
        uiDevice
            .findObjectWithTimeout(
                By.res(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_IMPORT_BUTTON_TEST_TAG)
            )
            .click()
    }
}
