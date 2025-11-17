package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import java.util.Locale
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
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

    fun scrollUntilWireGuardObfuscationQuicCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationLwoCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationOffCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG)
    }

    fun scrollUntilPostQuantumOffCell() {
        scrollUntilCell(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG)
    }

    fun scrollUntilPostQuantumOnCell() {
        scrollUntilCell(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG)
    }

    fun scrollUntilWireGuardObfuscationShadowsocksCell() {
        scrollUntilCell(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG)
    }

    fun scrollUntilWireGuardCustomPort() {
        scrollUntilCell(LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG)
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

    fun clickPostQuantumOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG))
    }

    fun clickPostQuantumOnCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG))
    }

    fun clickWireGuardObfuscationShadowsocksCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG))
    }

    private fun scrollUntilCell(testTag: String) {
        val scrollView2 = uiDevice.findObjectWithTimeout(By.res(LAZY_LIST_VPN_SETTINGS_TEST_TAG))
        scrollView2.scrollUntil(Direction.DOWN, Until.hasObject(By.res(testTag)))
    }

    fun clickWireguardCustomPort(port: Int? = null) {
        uiDevice
            .let {
                if (port != null) {
                    it.findObjectWithTimeout(
                        By.res(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG.format(Locale.US, port))
                    )
                } else {
                    it.findObjectWithTimeout(By.res(LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG))
                }
            }
            .click()
    }
}
