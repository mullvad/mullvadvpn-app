package net.mullvad.mullvadvpn.test.benchmark

import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.test.benchmark.rule.AccountTestRule
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class SpeedTest : BenchmarkTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    @Test
    fun noVpn() {
        val result = runIperf3(context)

        Logger.d("# No VPN result #")
        Logger.d(result.summarize())
    }

    @Test
    fun testNoObfuscation() {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickSettings() }
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationOffCell()
            clickWireGuardObfuscationOffCell()
        }
        device.pressBack()
        device.pressBack()

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(DEFAULT_COUNTRY)
            clickLocationExpandButton(DEFAULT_CITY)
            clickLocationCell(DEFAULT_RELAY)
        }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context)

        Logger.d("# No obfuscation #")
        Logger.d(result.summarize())
    }

    @Test
    fun testUdpOverTcpObfuscation() {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickSettings() }
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationUdpOverTcpCell()
            clickWireguardObfuscationUdpOverTcpCell()
        }
        device.pressBack()
        device.pressBack()

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context)

        Logger.d("# Udp-over-Tcp #")
        Logger.d(result.summarize())
    }

    @Test
    fun testShadowsocks() {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickSettings() }
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationShadowsocksCell()
            clickWireGuardObfuscationShadowsocksCell()
        }
        device.pressBack()
        device.pressBack()

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context)

        Logger.d("# Shadowsocks #")
        Logger.d(result.summarize())
    }
}

const val DEFAULT_COUNTRY = "Sweden"
const val DEFAULT_CITY = "Gothenburg"
const val DEFAULT_RELAY = "se-got-wg-001"
