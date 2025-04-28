package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.QUANTUM_RESISTANCE_OFF_CELL_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.ui.tag.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG

class VpnSettingsPage internal constructor() : Page() {
    private val vpnSettingsSelector = By.text("VPN settings")
    private val localNetworkSharingSelector = By.text("Local network sharing")

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

    fun scrollUntilWireGuardObfuscationUdpOverTcpCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationOffCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG)
    }

    fun scrollUntilPostQuantumOffCell() {
        scrollUntilCell(QUANTUM_RESISTANCE_OFF_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationShadowsocksCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG)
    }

    fun clickWireguardObfuscationUdpOverTcpCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG))
    }

    fun clickWireGuardObfuscationOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG))
    }

    fun clickPostQuantumOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(QUANTUM_RESISTANCE_OFF_CELL_TEST_TAG))
    }

    fun clickWireGuardObfuscationShadowsocksCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG))
    }

    private fun scrollUntilCell(testTag: String) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_VPN_SETTINGS_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(testTag)))
    }

    fun clickWireguardCustomPort() {
        uiDevice
            .findObjectWithTimeout(By.res(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
            .click()
    }
}
