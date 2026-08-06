package net.mullvad.mullvadvpn.test.e2e

import android.net.InetAddresses.parseNumericAddress
import androidx.test.uiautomator.waitForStableInActiveWindow
import java.net.Inet6Address
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.test.api.connectioncheck.ConnectionCheckApi
import net.mullvad.mullvadvpn.test.api.relay.RelayApi
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.interactor.DaitaOption
import net.mullvad.mullvadvpn.test.common.misc.RelayProvider
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.ObfuscationOption
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.disablePostQuantumStory
import net.mullvad.mullvadvpn.test.common.page.enableDeviceIpv6Story
import net.mullvad.mullvadvpn.test.common.page.enableLocalNetworkSharingStory
import net.mullvad.mullvadvpn.test.common.page.enableMultihopStory
import net.mullvad.mullvadvpn.test.common.page.enableWireGuardCustomPortStory
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.page.selectRelayUsingSearch
import net.mullvad.mullvadvpn.test.common.page.setObfuscationStory
import net.mullvad.mullvadvpn.test.common.page.toggleInTunnelIpv6Story
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.annotations.HasDependencyOnLocalAPI
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.LocalNetworkPermission
import net.mullvad.mullvadvpn.test.e2e.router.firewall.DropRule
import net.mullvad.mullvadvpn.test.e2e.router.firewall.FirewallClient
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertInstanceOf
import org.junit.jupiter.api.extension.RegisterExtension

class ConnectionTest1 : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    // Required on Android 17+ to allow access to the RASS router API.
    // It does not use the GrantPermissionExtension due to a crash.
    @RegisterExtension @JvmField val localNetworkPermission = LocalNetworkPermission()

    private val connCheckClient = ConnectionCheckApi(BuildConfig.INFRASTRUCTURE_BASE_DOMAIN)
    private val relayClient =
        RelayApi(
            billingFlavor = BuildConfig.FLAVOR_billing,
            baseDomain = BuildConfig.INFRASTRUCTURE_BASE_DOMAIN,
        )
    private val firewallClient by lazy { FirewallClient() }
    private val relayProvider = RelayProvider(BuildConfig.FLAVOR_billing)

    @Test
    @HasDependencyOnLocalAPI
    fun testApiUnavailable() = runTest {

        app.launchAndLogIn(accountTestRule.validAccountNumber)
        on<ConnectPage>()

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> { selectRelayUsingSearch(relayProvider.getDefaultRelay()) }

        device.acceptVpnPermissionDialog()

        // Test that we can still connect to the relay even though the API is blocked.
        on<ConnectPage> {
            waitForConnectedLabel()
            clickDisconnect()
            waitForDisconnectedLabel()
        }
    }

    @Test
    fun testConnectUsingMultihop() =
        runTest(timeout = 2.minutes) {
            // Given
            app.launchAndLogIn(accountTestRule.validAccountNumber)

            // Enable multihop
            on<ConnectPage> { enableMultihopStory() }

            // Select entry and exit relay
            on<ConnectPage> { clickSelectLocation() }
            val (entryRelay, exitRelay) = relayProvider.getMultihopRelays()
            on<SelectLocationPage> {
                // Select entry list
                clickEntryHopSelector()

                uiDevice.waitForStableInActiveWindow()

                // Select entry relay
                selectRelayUsingSearch(entryRelay)

                // Select exit relay
                selectRelayUsingSearch(exitRelay)
            }

            device.acceptVpnPermissionDialog()

            var outIpv4Address = ""
            on<ConnectPage> {
                waitForConnectedLabel()
                outIpv4Address = extractOutIpv4Address()
            }

            val result = connCheckClient.connectionCheck()

            // Check IPs match and that the out server is default server
            assertEquals(result.ip, outIpv4Address)
            assertEquals(result.mullvadExitIpHostname, exitRelay.relay)
        }


    companion object {
        const val VERY_FORGIVING_WIREGUARD_OFF_CONNECTION_TIMEOUT = 80000L
        const val UNSUCCESSFUL_CONNECTION_TIMEOUT = 30000L
        const val ANY_IPV4_ADDRESS = "0.0.0.0/0"
    }
}
