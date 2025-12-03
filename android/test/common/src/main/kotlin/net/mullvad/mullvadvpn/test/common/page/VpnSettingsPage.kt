package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class VpnSettingsPage internal constructor() : Page() {
    private val vpnSettingsSelector = By.text("VPN settings")
    private val localNetworkSharingSelector = By.text("Local network sharing")
    private val inTunnelIpv6Selector = By.text("In-tunnel IPv6")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(vpnSettingsSelector)
    }

    fun clickLocalNetworkSharingSwitch() {
        val localNetworkSharingCell =
            uiDevice.findObjectWithTimeout(localNetworkSharingSelector).parent
        val localNetworkSharingSwitch =
            localNetworkSharingCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        localNetworkSharingSwitch.click()
    }

    fun clickInTunnelIpv6Switch() {
        val inTunnelIpv6Cell = uiDevice.findObjectWithTimeout(inTunnelIpv6Selector).parent
        val inTunnelIpv6Switch = inTunnelIpv6Cell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        inTunnelIpv6Switch.click()
    }

    fun scrollUntilPostQuantumOffCell() {
        scrollUntilCell(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG)
    }

    fun scrollUntilPostQuantumOnCell() {
        scrollUntilCell(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG)
    }

    fun scrollUntilAntiCensorshipCell() {
        scrollUntilCell(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
    }

    fun clickPostQuantumOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))
    }

    fun clickPostQuantumOnCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG))
    }

    fun clickAntiCensorshipCell() {
        uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)).click()
    }

    private fun scrollUntilCell(testTag: String) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_VPN_SETTINGS_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(testTag)))
    }
}
