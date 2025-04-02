package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

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
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(SETTINGS_SCROLL_VIEW_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(testTag)))
    }

    fun clickWireguardCustomPort() {
        uiDevice.findObjectWithTimeout(By.res(WIREGUARD_CUSTOM_PORT_CELL_TEST_TAG)).click()
    }

    companion object {
        const val SETTINGS_SCROLL_VIEW_TEST_TAG = "lazy_list_vpn_settings_test_tag"
        const val WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG =
            "wireguard_obfuscation_udp_over_tcp_cell_test_tag"
        const val WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG =
            "wireguard_obfuscation_off_cell_test_tag"
        const val WIREGUARD_CUSTOM_PORT_CELL_TEST_TAG =
            "lazy_list_wireguard_custom_port_text_test_tag"
        const val WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG =
            "wireguard_obfuscation_shadowsocks_cell_test_tag"
        const val SWITCH_TEST_TAG = "switch_test_tag"
        const val QUANTUM_RESISTANCE_OFF_CELL_TEST_TAG = "lazy_list_quantum_item_off_test_tag"
    }
}
