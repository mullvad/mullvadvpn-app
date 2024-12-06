package net.mullvad.mullvadvpn.test.e2e

import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.SystemVpnConfigurationAlert
import net.mullvad.mullvadvpn.test.common.page.TopBar
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.annotations.HasDependencyOnLocalAPI
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.ClearFirewallRules
import net.mullvad.mullvadvpn.test.e2e.misc.ConnCheckState
import net.mullvad.mullvadvpn.test.e2e.misc.SimpleMullvadHttpClient
import net.mullvad.mullvadvpn.test.e2e.router.firewall.DropRule
import net.mullvad.mullvadvpn.test.e2e.router.firewall.FirewallClient
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ConnectionTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    private val firewallClient = FirewallClient()

    @Test
    fun testConnect() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickConnect() }

        on<SystemVpnConfigurationAlert> { clickOk() }

        on<ConnectPage> { waitForConnectedLabel() }
    }

    @Test
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickConnect() }

        on<SystemVpnConfigurationAlert> { clickOk() }

        var expectedConnectionState: ConnCheckState? = null

        on<ConnectPage> {
            waitForConnectedLabel()
            expectedConnectionState = ConnCheckState(true, extractOutIpv4Address())
        }

        // Then
        val result = SimpleMullvadHttpClient(targetContext).runConnectionCheck()
        assertEquals(expectedConnectionState, result)
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testWireGuardObfuscationAutomatic() = runBlocking {
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
        enableLocalNetworkSharing()

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(DEFAULT_COUNTRY)
            clickLocationExpandButton(DEFAULT_CITY)
            clickLocationCell(DEFAULT_RELAY)
        }

        on<SystemVpnConfigurationAlert> { clickOk() }

        var relayIpAddress: String? = null

        on<ConnectPage> {
            waitForConnectedLabel()
            relayIpAddress = extractInIpv4Address()
            clickDisconnect()
        }

        // Block UDP traffic to the relay
        val firewallRule = DropRule.blockUDPTrafficRule(relayIpAddress!!)
        firewallClient.createRule(firewallRule)

        on<ConnectPage> {
            clickConnect()
            // Currently it takes ~45 seconds to connect with wg obfuscation automatic and UDP
            // traffic blocked so we need to be very forgiving
            waitForConnectedLabel(timeout = VERY_FORGIVING_WIREGUARD_OFF_CONNECTION_TIMEOUT)
        }
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testWireGuardObfuscationOff() = runBlocking {
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
        enableLocalNetworkSharing()

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(DEFAULT_COUNTRY)
            clickLocationExpandButton(DEFAULT_CITY)
            clickLocationCell(DEFAULT_RELAY)
        }

        on<SystemVpnConfigurationAlert> { clickOk() }

        var relayIpAddress: String? = null

        on<ConnectPage> {
            waitForConnectedLabel()
            relayIpAddress = extractInIpv4Address()
            clickDisconnect()
        }

        // Block UDP traffic to the relay
        val firewallRule = DropRule.blockUDPTrafficRule(relayIpAddress!!)
        firewallClient.createRule(firewallRule)

        // Enable UDP-over-TCP
        on<TopBar> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> {
            scrollUntilWireguardObfuscationOffCell()
            clickWireguardObfuscationOffCell()
        }

        device.pressBack()
        device.pressBack()

        on<ConnectPage> {
            clickConnect() // Ensure it is not possible to connect to relay
            // Give it some time and then verify still unable to connect. This duration must be long
            // enough to ensure all retry attempts have been made.
            delay(UNSUCCESSFUL_CONNECTION_TIMEOUT.milliseconds)
            waitForConnectingLabel()
            clickCancel()
        }
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testUDPOverTCP() =
        runBlocking<Unit> {
            app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
            enableLocalNetworkSharing()

            on<ConnectPage> { clickSelectLocation() }

            on<SelectLocationPage> {
                clickLocationExpandButton(DEFAULT_COUNTRY)
                clickLocationExpandButton(DEFAULT_CITY)
                clickLocationCell(DEFAULT_RELAY)
            }

            on<SystemVpnConfigurationAlert> { clickOk() }

            var relayIpAddress: String? = null

            on<ConnectPage> {
                waitForConnectedLabel()
                relayIpAddress = extractInIpv4Address()
                clickDisconnect()
            }

            // Block UDP traffic to the relay
            val firewallRule = DropRule.blockUDPTrafficRule(relayIpAddress!!)
            firewallClient.createRule(firewallRule)

            // Enable UDP-over-TCP
            on<TopBar> { clickSettings() }

            on<SettingsPage> { clickVpnSettings() }

            on<VpnSettingsPage> {
                scrollUntilWireguardObfuscationUdpOverTcpCell()
                clickWireguardObfuscationUdpOverTcpCell()
            }

            device.pressBack()
            device.pressBack()

            on<ConnectPage> {
                clickConnect()
                waitForConnectedLabel(timeout = EXTREMELY_LONG_TIMEOUT)
                clickDisconnect()
            }
        }

    private fun enableLocalNetworkSharing() {
        on<TopBar> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> { clickLocalNetworkSharingSwitch() }

        device.pressBack()
        device.pressBack()
    }

    companion object {
        const val VERY_FORGIVING_WIREGUARD_OFF_CONNECTION_TIMEOUT = 60000L
        const val UNSUCCESSFUL_CONNECTION_TIMEOUT = 60000L
    }
}
