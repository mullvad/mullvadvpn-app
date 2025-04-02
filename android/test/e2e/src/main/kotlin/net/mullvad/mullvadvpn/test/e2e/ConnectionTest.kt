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
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage
import net.mullvad.mullvadvpn.test.common.page.disableObfuscationStory
import net.mullvad.mullvadvpn.test.common.page.enableLocalNetworkSharingStory
import net.mullvad.mullvadvpn.test.common.page.enableShadowsocksStory
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
        on<ConnectPage> { enableLocalNetworkSharingStory() }

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
        on<ConnectPage> { enableLocalNetworkSharingStory() }

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
        on<ConnectPage> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationOffCell()
            clickWireGuardObfuscationOffCell()
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
            on<ConnectPage> { enableLocalNetworkSharingStory() }

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
            on<ConnectPage> { clickSettings() }

            on<SettingsPage> { clickVpnSettings() }

            on<VpnSettingsPage> {
                scrollUntilWireGuardObfuscationUdpOverTcpCell()
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

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testShadowsocks() =
        runBlocking<Unit> {
            app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
            on<ConnectPage> { enableLocalNetworkSharingStory() }

            on<ConnectPage> { disableObfuscationStory() }

            // Block all WireGuard traffic
            val firewallRule = DropRule.blockWireGuardTrafficRule(ANY_IP_ADDRESS)
            firewallClient.createRule(firewallRule)

            on<ConnectPage> { clickConnect() }

            on<SystemVpnConfigurationAlert> { clickOk() }

            // Ensure it is not possible to connect to relay
            on<ConnectPage> {
                delay(UNSUCCESSFUL_CONNECTION_TIMEOUT.milliseconds)
                waitForConnectingLabel()
                clickCancel()
            }

            on<ConnectPage> { enableShadowsocksStory() }

            // Ensure we can now connect with Shadowsocks enabled
            on<ConnectPage> {
                clickConnect()
                waitForConnectedLabel(timeout = EXTREMELY_LONG_TIMEOUT)
                clickDisconnect()
            }
        }

    companion object {
        const val VERY_FORGIVING_WIREGUARD_OFF_CONNECTION_TIMEOUT = 60000L
        const val UNSUCCESSFUL_CONNECTION_TIMEOUT = 60000L
        const val ANY_IP_ADDRESS = "0.0.0.0/0"
    }
}
