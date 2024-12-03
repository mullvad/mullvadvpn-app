package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.Until
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.test.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_SETTINGS_BUTTON
import net.mullvad.mullvadvpn.test.common.constant.EXTREMELY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
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

    val firewallClient = FirewallClient()

    @Test
    fun testConnect() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Connect")).click()
        device.findObjectWithTimeout(By.text("OK")).click()

        // Then
        device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)
    }

    @Test
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Connect")).click()
        device.findObjectWithTimeout(By.text("OK")).click()
        device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)
        val expected = ConnCheckState(true, app.extractOutIpv4Address())

        // Then
        val result = SimpleMullvadHttpClient(targetContext).runConnectionCheck()
        assertEquals(expected, result)
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testWireGuardObfuscationOff() = runBlocking {
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        enableLocalNetworkSharing()

        device.findObjectWithTimeout(By.res(SELECT_LOCATION_BUTTON_TEST_TAG)).click()
        clickLocationExpandButton(DEFAULT_COUNTRY)
        clickLocationExpandButton(DEFAULT_CITY)
        device.findObjectWithTimeout(By.text(DEFAULT_RELAY)).click()
        device.findObjectWithTimeout(By.text("OK")).click()
        device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)
        val relayIpAddress = app.extractInIpv4Address()
        device.findObjectWithTimeout(By.text("Disconnect")).click()

        // Disable obfuscation
        device.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
        device.findObjectWithTimeout(By.text("VPN settings")).click()
        val scrollView = device.findObjectWithTimeout(By.res(SETTINGS_SCROLL_VIEW_TEST_TAG))
        scrollView.scrollUntil(
            Direction.DOWN,
            Until.hasObject(By.res(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG)),
        )
        device.findObjectWithTimeout(By.res(WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG)).click()
        device.pressBack()
        device.pressBack()

        // Block UDP traffic to the relay
        val firewallRule = DropRule.blockUDPTrafficRule(relayIpAddress)
        firewallClient.createRule(firewallRule)

        // Ensure it is not possible to connect to relay
        device.findObjectWithTimeout(By.text("Connect")).click()
        // Give it some time and then verify still unable to connect. This duration must be long
        // enough to ensure all retry attempts have been made.
        delay(UNSUCCESSFUL_CONNECTION_TIMEOUT.milliseconds)
        device.findObjectWithTimeout(By.text(("CONNECTING...")))
        device.findObjectWithTimeout(By.text("Cancel")).click()
    }

    @Test
    @HasDependencyOnLocalAPI
    @ClearFirewallRules
    fun testUDPOverTCP() =
        runBlocking<Unit> {
            app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

            enableLocalNetworkSharing()

            device.findObjectWithTimeout(By.res(SELECT_LOCATION_BUTTON_TEST_TAG)).click()
            clickLocationExpandButton(DEFAULT_COUNTRY)
            clickLocationExpandButton(DEFAULT_CITY)
            device.findObjectWithTimeout(By.text(DEFAULT_RELAY)).click()
            device.findObjectWithTimeout(By.text("OK")).click()
            device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)
            val relayIpAddress = app.extractInIpv4Address()
            device.findObjectWithTimeout(By.text("Disconnect")).click()

            // Block UDP traffic to the relay
            val firewallRule = DropRule.blockUDPTrafficRule(relayIpAddress)
            firewallClient.createRule(firewallRule)

            // Enable UDP-over-TCP
            device.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
            device.findObjectWithTimeout(By.text("VPN settings")).click()
            val scrollView2 = device.findObjectWithTimeout(By.res(SETTINGS_SCROLL_VIEW_TEST_TAG))
            scrollView2.scrollUntil(
                Direction.DOWN,
                Until.hasObject(By.res(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG)),
            )
            device
                .findObjectWithTimeout(By.res(WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG))
                .click()
            device.pressBack()
            device.pressBack()

            // Ensure it is possible to connect by using UDP-over-TCP
            device.findObjectWithTimeout(By.text("Connect")).click()
            device.findObjectWithTimeout(By.text("CONNECTED"), EXTREMELY_LONG_TIMEOUT)
            device.findObjectWithTimeout(By.text("Disconnect")).click()
        }

    private fun enableLocalNetworkSharing() {
        device.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
        device.findObjectWithTimeout(By.text("VPN settings")).click()

        val localNetworkSharingCell =
            device.findObjectWithTimeout(By.text("Local network sharing")).parent
        val localNetworkSharingSwitch =
            localNetworkSharingCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        localNetworkSharingSwitch.click()
        device.pressBack()
        device.pressBack()
    }

    private fun clickLocationExpandButton(locationName: String) {
        val locationCell = device.findObjectWithTimeout(By.text(locationName)).parent.parent
        val expandButton = locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
        expandButton.click()
    }

    companion object {
        const val SETTINGS_SCROLL_VIEW_TEST_TAG = "lazy_list_vpn_settings_test_tag"
        const val WIREGUARD_OBFUSCATION_OFF_CELL_TEST_TAG =
            "wireguard_obfuscation_off_cell_test_tag"
        const val WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL_TEST_TAG =
            "wireguard_obfuscation_udp_over_tcp_cell_test_tag"
        const val UNSUCCESSFUL_CONNECTION_TIMEOUT = 60000L
    }
}
