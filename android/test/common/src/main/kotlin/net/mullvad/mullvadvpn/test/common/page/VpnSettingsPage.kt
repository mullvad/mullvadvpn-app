package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV4_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV6_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class VpnSettingsPage internal constructor() : Page() {
    private val vpnSettingsSelector = By.text("VPN settings")
    private val localNetworkSharingSelector = By.text("Local network sharing")
    private val inTunnelIpv6Selector = By.text("In-tunnel IPv6")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(vpnSettingsSelector)
    }

    fun assertPostQuantumState(enabled: Boolean) {
        val postQuantumCell =
            uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))
        val postQuantumSwitch = postQuantumCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        assert(postQuantumSwitch.isChecked == enabled)
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

    fun scrollUntilWireGuardObfuscationUdpOverTcpCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationQuicCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationLwoCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationOffCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG)
    }

    fun scrollUntilPostQuantumCell() {
        scrollUntilCell(LAZY_LIST_QUANTUM_ITEM_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationShadowsocksCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardCustomPort() {
        scrollUntilCell(LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG)
    }

    fun scrollUntilServerIpOverride() {
        scrollUntilCell(SERVER_IP_OVERRIDE_BUTTON_TEST_TAG)
    }

    fun scrollUntilDeviceIpVersionCell() {
        scrollUntilCell(WIREGUARD_DEVICE_IP_IPV6_CELL_TEST_TAG)
    }

    fun clickWireguardObfuscationUdpOverTcpCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG))
    }

    fun clickWireguardObfuscationQuicCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG))
    }

    fun clickWireguardObfuscationLwoCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG))
    }

    fun clickWireGuardObfuscationOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG))
    }

    fun scrollUntilAntiCensorshipCell() {
        scrollUntilCell(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
    }

    fun clickPostQuantumCell() {
        val postQuantumCell =
            uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_QUANTUM_ITEM_TEST_TAG))
        val postQuantumSwitch = postQuantumCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        postQuantumSwitch.click()
    }

    fun clickAntiCensorshipCell() {
        uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)).click()
    }

    fun clickServerIpOverrideButton() {
        uiDevice.findObjectWithTimeout(By.res(SERVER_IP_OVERRIDE_BUTTON_TEST_TAG)).click()
    }

    fun clickDeviceIpIpv4Cell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_DEVICE_IP_IPV4_CELL_TEST_TAG))
    }

    private fun scrollUntilCell(testTag: String) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_VPN_SETTINGS_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(testTag)))
    }
}
