package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

abstract class Page(val device: UiDevice, val pageSelector: BySelector) {
    init {
        verifyPageShown()
    }

    private fun verifyPageShown() {
        device.findObjectWithTimeout(pageSelector)
    }
}
