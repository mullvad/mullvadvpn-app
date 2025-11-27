package net.mullvad.mullvadvpn.test.e2e

import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage
import net.mullvad.mullvadvpn.test.common.page.disableObfuscationStory
import net.mullvad.mullvadvpn.test.common.page.enableLocalNetworkSharingStory
import net.mullvad.mullvadvpn.test.common.page.enablePostQuantumStory
import net.mullvad.mullvadvpn.test.common.page.enableServerIpOverrideStory
import net.mullvad.mullvadvpn.test.common.page.enableShadowsocksStory
import net.mullvad.mullvadvpn.test.common.page.enableWireGuardCustomPort
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.annotations.HasDependencyOnLocalAPI
import net.mullvad.mullvadvpn.test.e2e.api.connectioncheck.ConnectionCheckApi
import net.mullvad.mullvadvpn.test.e2e.api.relay.RelayApi
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.ClearFirewallRules
import net.mullvad.mullvadvpn.test.e2e.misc.RelayProvider
import net.mullvad.mullvadvpn.test.e2e.router.firewall.DropRule
import net.mullvad.mullvadvpn.test.e2e.router.firewall.FirewallClient
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ConnectionTest : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    private val connCheckClient = ConnectionCheckApi()
    private val relayClient = RelayApi()
    private val firewallClient by lazy { FirewallClient() }
    private val relayProvider = RelayProvider()

    @Test
    fun testConnect() {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }
    }

    @Test
    fun testConnectAndVerifyWithConnectionCheck() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        var outIpv4Address = ""

        on<ConnectPage> {
            waitForConnectedLabel()
            outIpv4Address = extractOutIpv4Address()
        }

        // Then
        val result = connCheckClient.connectionCheck()

        assertEquals(result.ip, outIpv4Address)
    }

    @Test
    fun testConnectUsingPostQuantum() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        // Enable post quantum
        on<ConnectPage> { enablePostQuantumStory() }

        // Connect
        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        var outIpv4Address = ""

        on<ConnectPage> {
            waitForConnectedLabel()
            outIpv4Address = extractOutIpv4Address()
        }

        val result = connCheckClient.connectionCheck()

        // Verify connection
        assertEquals(result.ip, outIpv4Address)
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testWireGuardObfuscationAutomatic() =
        runTest(timeout = 2.minutes) {
            app.launchAndLogIn(accountTestRule.validAccountNumber)
            on<ConnectPage> { enableLocalNetworkSharingStory() }

            on<ConnectPage> { clickSelectLocation() }

            on<SelectLocationPage> {
                clickLocationExpandButton(relayProvider.getDefaultRelay().country)
                clickLocationExpandButton(relayProvider.getDefaultRelay().city)
                clickLocationCell(relayProvider.getDefaultRelay().relay)
            }

            device.acceptVpnPermissionDialog()

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
    fun testWireGuardObfuscationOff() =
        runTest(timeout = 2.minutes) {
            app.launchAndLogIn(accountTestRule.validAccountNumber)
            on<ConnectPage> { enableLocalNetworkSharingStory() }

            on<ConnectPage> { clickSelectLocation() }

            on<SelectLocationPage> {
                clickLocationExpandButton(relayProvider.getDefaultRelay().country)
                clickLocationExpandButton(relayProvider.getDefaultRelay().city)
                clickLocationCell(relayProvider.getDefaultRelay().relay)
            }

            device.acceptVpnPermissionDialog()

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
                // Give it some time and then verify still unable to connect. This duration must be
                // long
                // enough to ensure all retry attempts have been made.
                runBlocking { delay(UNSUCCESSFUL_CONNECTION_TIMEOUT.milliseconds) }
                waitForConnectingLabel()
                clickCancel()
            }
        }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testUDPOverTCP() = runTest {
        app.launchAndLogIn(accountTestRule.validAccountNumber)
        on<ConnectPage> { enableLocalNetworkSharingStory() }

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(relayProvider.getDefaultRelay().country)
            clickLocationExpandButton(relayProvider.getDefaultRelay().city)
            clickLocationCell(relayProvider.getDefaultRelay().relay)
        }

        device.acceptVpnPermissionDialog()

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
    fun testQuic() = runTest {
        app.launchAndLogIn(accountTestRule.validAccountNumber)
        on<ConnectPage> { enableLocalNetworkSharingStory() }

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            val quicRelay = relayProvider.getQuicRelay()
            clickLocationExpandButton(quicRelay.country)
            clickLocationExpandButton(quicRelay.city)
            scrollUntilCell(quicRelay.relay)
            clickLocationCell(quicRelay.relay)
        }

        device.acceptVpnPermissionDialog()

        var relayIpAddress: String? = null

        on<ConnectPage> {
            waitForConnectedLabel()
            relayIpAddress = extractInIpv4Address()
            clickDisconnect()
        }

        // Block UDP traffic to the relay
        val firewallRule = DropRule.blockWireGuardTrafficRule(relayIpAddress!!)
        firewallClient.createRule(firewallRule)

        // Enable QUIC
        on<ConnectPage> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationQuicCell()
            clickWireguardObfuscationQuicCell()
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
    fun testLwo() = runTest {
        app.launchAndLogIn(accountTestRule.validAccountNumber)
        on<ConnectPage> { enableLocalNetworkSharingStory() }

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            val lwoRelay = relayProvider.getLwoRelay()
            clickLocationExpandButton(lwoRelay.country)
            clickLocationExpandButton(lwoRelay.city)
            scrollUntilCell(lwoRelay.relay)
            clickLocationCell(lwoRelay.relay)
        }

        device.acceptVpnPermissionDialog()

        var relayIpAddress: String? = null

        on<ConnectPage> {
            waitForConnectedLabel()
            relayIpAddress = extractInIpv4Address()
            clickDisconnect()
        }

        // Block UDP traffic to the relay
        val firewallRule = DropRule.blockWireGuardTrafficRule(relayIpAddress!!)
        firewallClient.createRule(firewallRule)

        // Enable QUIC
        on<ConnectPage> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationLwoCell()
            clickWireguardObfuscationLwoCell()
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
        runTest(timeout = 2.minutes) {
            app.launchAndLogIn(accountTestRule.validAccountNumber)
            on<ConnectPage> { enableLocalNetworkSharingStory() }

            on<ConnectPage> { disableObfuscationStory() }

            // Block all WireGuard traffic
            val firewallRule = DropRule.blockWireGuardTrafficRule(ANY_IP_ADDRESS)
            firewallClient.createRule(firewallRule)

            on<ConnectPage> { clickConnect() }

            device.acceptVpnPermissionDialog()

            // Ensure it is not possible to connect to relay
            on<ConnectPage> {
                runBlocking { delay(UNSUCCESSFUL_CONNECTION_TIMEOUT.milliseconds) }
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

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testApiUnavailable() = runTest {
        val testRelayIp = relayClient.getDefaultRelayIpAddress()

        app.launchAndLogIn(accountTestRule.validAccountNumber)
        on<ConnectPage>()

        // Block everything except the default relay IP. After this the API is no longer reachable.
        val firewallRule = DropRule.blockAllTrafficExceptToDestinationRule(testRelayIp)
        firewallClient.createRule(firewallRule)

        // Restarting the activity will re-create the daemon which will try to reach the API.
        targetActivity.finishAffinity()
        app.launch()

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(relayProvider.getDefaultRelay().country)
            clickLocationExpandButton(relayProvider.getDefaultRelay().city)
            clickLocationCell(relayProvider.getDefaultRelay().relay)
        }

        device.acceptVpnPermissionDialog()

        // Test that we can still connect to the relay even though the API is blocked.
        on<ConnectPage> {
            waitForConnectedLabel()
            clickDisconnect()
            waitForDisconnectedLabel()
        }
    }

    @Test
    fun testConnectUsingWireguardCustomPort() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        // Set wireguard custom port
        on<ConnectPage> { enableWireGuardCustomPort(53) }

        // Connect
        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        var inIpv4Port = ""

        on<ConnectPage> {
            waitForConnectedLabel()
            inIpv4Port = extractInIpv4Port()
        }

        // Verify correct port used
        assertEquals("53", inIpv4Port)
    }

    @Test
    fun testAttemptToConnectUsingServerIpOverride() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        // Enable server ip override
        val mockServerIp = "12.12.12.12"
        val relay = relayProvider.getDefaultRelay().relay
        on<ConnectPage> { enableServerIpOverrideStory(relay, mockServerIp) }

        // Select the relay which has an overriden ip
        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(relayProvider.getDefaultRelay().country)
            clickLocationExpandButton(relayProvider.getDefaultRelay().city)
            clickLocationCell(relayProvider.getDefaultRelay().relay)
        }

        device.acceptVpnPermissionDialog()

        var inIpv4Address = ""

        on<ConnectPage> { inIpv4Address = extractInIpv4Address() }

        // Verify connection
        assertEquals(mockServerIp, inIpv4Address)
    }

    companion object {
        const val VERY_FORGIVING_WIREGUARD_OFF_CONNECTION_TIMEOUT = 60000L
        const val UNSUCCESSFUL_CONNECTION_TIMEOUT = 30000L
        const val ANY_IP_ADDRESS = "0.0.0.0/0"
    }
}
