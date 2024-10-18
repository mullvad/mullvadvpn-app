package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice

class ConnectPage(device: UiDevice) : Page(device, pageSelector = By.res("connect_card_header_test_tag")) {
    // No connect page functions needed for this POC
}
