package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class WireGuardCustomPortDialog internal constructor() : Page() {
    private val textFieldLabelSelector = By.text("Enter port")
    private val setPortSelector = By.text("Set port")
    private val cancelSelector = By.text("Cancel")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(textFieldLabelSelector)
    }

    fun enterCustomPort(port: String) {
        uiDevice.findObjectWithTimeout(textFieldLabelSelector).parent.text = port
    }

    fun clickSetPort() {
        uiDevice.findObjectWithTimeout(setPortSelector).click()
    }

    fun clickCancel() {
        uiDevice.findObjectWithTimeout(cancelSelector).click()
    }

    companion object {
        const val TEXT_FIELD_TEST_TAG = "custom_port_dialog_input_test_tag"
    }
}
