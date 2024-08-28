package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCapture
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCaptureClient
import net.mullvad.mullvadvpn.test.e2e.misc.TrafficGenerator
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LeakTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    val packetCaptureClient = PacketCaptureClient()

    @Test
    fun testNegativeLeak() =
        runBlocking<Unit> {
            val targetIpAddress = "45.83.223.209"
            val packetCapture = PacketCapture()
            val trafficGenerator = TrafficGenerator(targetIpAddress, 80)

            val session = packetCapture.startCapture()
            packetCaptureClient.sendStartCaptureRequest(session)
            trafficGenerator.startGeneratingUDPTraffic(100)

            // Given
            app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

            // When
            device.findObjectWithTimeout(By.text("Secure my connection")).click()
            device.findObjectWithTimeout(By.text("OK")).click()

            // Then
            device.findObjectWithTimeout(By.text("SECURE CONNECTION"), CONNECTION_TIMEOUT)
            val relayIpAddress = app.extractInIpAddress()

            device.findObjectWithTimeout(By.text("Disconnect")).click()
            Thread.sleep(2000)

            trafficGenerator.stopGeneratingUDPTraffic()
            packetCaptureClient.sendStopCaptureRequest(session)
            val streamCollection = packetCapture.stopCapture(session)
            val connectedThroughRelayStartEndDatePair =
                streamCollection.getConnectedThroughRelayStartEndDate(relayIpAddress)

            // Verify that all traffic to target IP address went through relay while VPN connection
            // was active
        }
}
