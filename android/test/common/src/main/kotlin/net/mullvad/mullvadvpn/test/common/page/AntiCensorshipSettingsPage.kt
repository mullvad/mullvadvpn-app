package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.BUTTON_ARROW_RIGHT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.VPN_SETTINGS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.clickObjectAwaitIsChecked
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class AntiCensorshipSettingsPage internal constructor() : Page() {
    private val settingsSelector = By.text("Anti-censorship")
    private val faqAndGuidesSelector = By.text("FAQs & Guides")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(settingsSelector)
    }

    fun clickVpnSettings() {
        uiDevice.findObjectWithTimeout(By.res(VPN_SETTINGS_CELL_TEST_TAG)).click()
    }

    fun clickFaqAndGuides() {
        uiDevice.findObjectWithTimeout(faqAndGuidesSelector).click()
    }

    fun clickDaita() {
        uiDevice.findObjectWithTimeout(By.res(DAITA_CELL_TEST_TAG)).click()
    }

    fun clickWireguardSelectPortButton() {
        uiDevice
            .findObjectWithTimeout(By.res(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG))
            .parent
            .findObject(By.res(BUTTON_ARROW_RIGHT_TEST_TAG))
            .click()
    }

    fun clickWireguardPortCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_WG_PORT_TEST_TAG))
    }

    fun clickShadowsocksCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL_TEST_TAG))
    }

    fun clickUdp2TcpCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG))
    }

    fun clickObfuscationOffCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG))
    }

    fun clickQuicCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_QUIC_CELL_TEST_TAG))
    }

    fun clickLwoCell() {
        uiDevice.clickObjectAwaitIsChecked(By.res(WIREGUARD_OBFUSCATION_LWO_CELL_TEST_TAG))
    }
}
