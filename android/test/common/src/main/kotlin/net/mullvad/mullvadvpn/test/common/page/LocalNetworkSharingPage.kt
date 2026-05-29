package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.LOCAL_NETWORK_SHARING_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class LocalNetworkSharingPage internal constructor() : Page() {
    private val localNetworkSharingSelector = By.res(LOCAL_NETWORK_SHARING_SCREEN_TEST_TAG)

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(localNetworkSharingSelector)
    }

    fun toggleSwitch() {
        val switch = uiDevice.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))
        switch.click()
    }
}
