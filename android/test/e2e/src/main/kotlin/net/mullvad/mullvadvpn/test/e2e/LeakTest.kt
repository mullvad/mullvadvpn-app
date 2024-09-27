package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.test.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_SETTINGS_BUTTON
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.misc.Attachment
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.LeakCheck
import net.mullvad.mullvadvpn.test.e2e.misc.NoTrafficToHostRule
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCapture
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCaptureResult
import net.mullvad.mullvadvpn.test.e2e.misc.TrafficGenerator
import org.joda.time.DateTime
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LeakTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    @BeforeEach
    fun setupVPNSettings() {
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
        device.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
        device.findObjectWithTimeout(By.text("VPN settings")).click()

        val localNetworkSharingCell =
            device.findObjectWithTimeout(By.text("Local network sharing")).parent
        val localNetworkSharingSwitch =
            localNetworkSharingCell.findObjectWithTimeout(By.res(SWITCH_TEST_TAG))

        if (localNetworkSharingSwitch.isChecked.not()) {
            localNetworkSharingSwitch.click()
        }

        // Only use port 51820 to make packet capture more deterministic
        device.findObjectWithTimeout(By.text("51820")).click()

        device.pressBack()
        device.pressBack()
    }

    @Test
    fun testNegativeLeak() =
        runBlocking<Unit> {
            app.launch()
            device.findObjectWithTimeout(By.text("DISCONNECTED"))

            val targetIpAddress = "45.83.223.209"
            val targetPort = 80

            device.findObjectWithTimeout(By.res(SELECT_LOCATION_BUTTON_TEST_TAG)).click()
            clickLocationExpandButton((EndToEndTest.DEFAULT_COUNTRY))
            clickLocationExpandButton((EndToEndTest.DEFAULT_CITY))
            device.findObjectWithTimeout(By.text(EndToEndTest.DEFAULT_RELAY)).click()
            device.findObjectWithTimeout(By.text("OK")).click()
            device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)

            val connectTime = DateTime.now()
            val captureResult =
                PacketCapture().capturePackets {
                    TrafficGenerator(targetIpAddress, targetPort).generateTraffic(10.milliseconds) {
                        // Give it some time for generating traffic
                        delay(3000)
                    }
                }

            val disconnectTime = DateTime.now()
            device.findObjectWithTimeout(By.text("Disconnect")).click()

            val capturedStreams = captureResult.streams
            val capturedPcap = captureResult.pcap

            val timestamp = System.currentTimeMillis()
            Attachment.saveAttachment("capture-testNegativeLeak-$timestamp.pcap", capturedPcap)

            val leakRules = listOf(NoTrafficToHostRule(targetIpAddress))

            LeakCheck.assertNoLeaks(capturedStreams, leakRules, connectTime, disconnectTime)
        }

    @Test
    fun testShouldHaveNegativeLeak() =
        runBlocking<Unit> {
            app.launch()
            device.findObjectWithTimeout(By.text("DISCONNECTED"))

            val targetIpAddress = "45.83.223.209"
            val targetPort = 80

            device.findObjectWithTimeout(By.res(SELECT_LOCATION_BUTTON_TEST_TAG)).click()
            delay(1000.milliseconds)
            clickLocationExpandButton((EndToEndTest.DEFAULT_COUNTRY))
            clickLocationExpandButton((EndToEndTest.DEFAULT_CITY))
            device.findObjectWithTimeout(By.text(EndToEndTest.DEFAULT_RELAY)).click()
            device.findObjectWithTimeout(By.text("OK")).click()
            device.findObjectWithTimeout(By.text("CONNECTED"), VERY_LONG_TIMEOUT)

            lateinit var captureResult: PacketCaptureResult
            val connectTime = DateTime.now()

            TrafficGenerator(targetIpAddress, targetPort).generateTraffic(10.milliseconds) {
                captureResult =
                    PacketCapture().capturePackets {
                        delay(
                            3000.milliseconds
                        ) // Give it some time for generating traffic in tunnel
                        device.findObjectWithTimeout(By.text("Disconnect")).click()
                        delay(
                            2000.milliseconds
                        ) // Give it some time to leak traffic outside of tunnel
                        device.findObjectWithTimeout(By.text("Connect")).click()
                        delay(
                            3000.milliseconds
                        ) // Give it some time for generating traffic in tunnel
                    }
            }

            val disconnectTime = DateTime.now()
            device.findObjectWithTimeout(By.text("Disconnect")).click()

            val capturedStreams = captureResult.streams
            val capturedPcap = captureResult.pcap
            val timestamp = System.currentTimeMillis()
            Attachment.saveAttachment("capture-testShouldHaveLeak-$timestamp.pcap", capturedPcap)

            val leakRules = listOf(NoTrafficToHostRule(targetIpAddress))

            LeakCheck.assertLeaks(capturedStreams, leakRules, connectTime, disconnectTime)
        }

    private fun clickLocationExpandButton(locationName: String) {
        val locationCell = device.findObjectWithTimeout(By.text(locationName)).parent.parent
        val expandButton = locationCell.findObjectWithTimeout(By.res(EXPAND_BUTTON_TEST_TAG))
        expandButton.click()
    }
}
