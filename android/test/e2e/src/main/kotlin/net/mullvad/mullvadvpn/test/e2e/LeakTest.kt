package net.mullvad.mullvadvpn.test.e2e

import androidx.test.platform.app.InstrumentationRegistry
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlinx.coroutines.delay
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.misc.Attachment
import net.mullvad.mullvadvpn.test.common.misc.RelayProvider
import net.mullvad.mullvadvpn.test.common.page.AntiCensorshipSettingsPage
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.ObfuscationOption
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.SelectPortPage
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.VpnSettingsPage
import net.mullvad.mullvadvpn.test.common.page.enableDAITAStory
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.page.setObfuscationStory
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.annotations.HasDependencyOnLocalAPI
import net.mullvad.mullvadvpn.test.e2e.constant.getTrafficGeneratorHost
import net.mullvad.mullvadvpn.test.e2e.constant.getTrafficGeneratorPort
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.NetworkTrafficChecker
import net.mullvad.mullvadvpn.test.e2e.misc.NoTrafficToHostRule
import net.mullvad.mullvadvpn.test.e2e.misc.SomeTrafficToHostRule
import net.mullvad.mullvadvpn.test.e2e.misc.SomeTrafficToOtherHostsRule
import net.mullvad.mullvadvpn.test.e2e.misc.TrafficGenerator
import net.mullvad.mullvadvpn.test.e2e.router.packetCapture.PacketCapture
import net.mullvad.mullvadvpn.test.e2e.router.packetCapture.PacketCaptureResult
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LeakTest : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    val relayProvider = RelayProvider(BuildConfig.FLAVOR_billing)

    @BeforeEach
    fun setupVPNSettings() {
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        on<ConnectPage> { clickSettings() }

        on<SettingsPage> { clickVpnSettings() }

        on<VpnSettingsPage> {
            clickLocalNetworkSharingSwitch()
            scrollUntilAntiCensorshipCell()
            clickAntiCensorshipCell()
        }

        on<AntiCensorshipSettingsPage> { clickWireguardSelectPortButton() }

        on<SelectPortPage> { clickPresetPort(51820) }

        device.pressBack()

        on<AntiCensorshipSettingsPage> { clickWireguardPortCell() }

        repeat(3) { device.pressBack() }
    }

    @Test
    @HasDependencyOnLocalAPI
    fun testEnsureNoLeaksToSpecificHost() = runTest {
        app.launch()

        on<ConnectPage> {
            waitForDisconnectedLabel()

            clickSelectLocation()
        }

        on<SelectLocationPage> {
            clickLocationExpandButton(relayProvider.getDefaultRelay().country)
            clickLocationExpandButton(relayProvider.getDefaultRelay().city)
            clickLocationCell(relayProvider.getDefaultRelay().relay)
        }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        // Capture generated traffic to a specific host
        val targetIpAddress = InstrumentationRegistry.getArguments().getTrafficGeneratorHost()
        val targetPort = InstrumentationRegistry.getArguments().getTrafficGeneratorPort()
        val captureResult =
            PacketCapture().capturePackets {
                TrafficGenerator(targetIpAddress, targetPort).generateTraffic(10.milliseconds) {
                    // Give it some time for generating traffic
                    delay(3000)
                }
            }

        on<ConnectPage> { clickDisconnect() }

        val capturedStreams = captureResult.streams
        val capturedPcap = captureResult.pcap
        val timestamp = System.currentTimeMillis()
        Attachment.saveAttachment(
            "capture-${javaClass.enclosingMethod}-$timestamp.pcap",
            capturedPcap,
        )

        NetworkTrafficChecker.checkTrafficStreamsAgainstRules(
            capturedStreams,
            NoTrafficToHostRule(targetIpAddress),
        )
    }

    @Test
    @HasDependencyOnLocalAPI
    fun testEnsureLeaksToSpecificHost() = runTest {
        app.launch()

        on<ConnectPage> {
            waitForDisconnectedLabel()

            clickSelectLocation()
        }

        on<SelectLocationPage> {
            clickLocationExpandButton(relayProvider.getDefaultRelay().country)
            clickLocationExpandButton(relayProvider.getDefaultRelay().city)
            clickLocationCell(relayProvider.getDefaultRelay().relay)
        }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        // Capture generated traffic to a specific host
        val targetIpAddress = InstrumentationRegistry.getArguments().getTrafficGeneratorHost()
        val targetPort = InstrumentationRegistry.getArguments().getTrafficGeneratorPort()
        val captureResult: PacketCaptureResult =
            PacketCapture().capturePackets {
                TrafficGenerator(targetIpAddress, targetPort).generateTraffic(10.milliseconds) {
                    delay(3000.milliseconds) // Give it some time for generating traffic in tunnel

                    on<ConnectPage> { clickDisconnect() }

                    delay(2000.milliseconds) // Give it some time to leak traffic outside of tunnel

                    on<ConnectPage> {
                        clickConnect()
                        waitForConnectedLabel()
                    }

                    delay(3000.milliseconds) // Give it some time for generating traffic in tunnel
                }
            }

        on<ConnectPage> { clickDisconnect() }

        val capturedStreams = captureResult.streams
        val capturedPcap = captureResult.pcap
        val timestamp = System.currentTimeMillis()
        Attachment.saveAttachment(
            "capture-${javaClass.enclosingMethod}-$timestamp.pcap",
            capturedPcap,
        )

        NetworkTrafficChecker.checkTrafficStreamsAgainstRules(
            capturedStreams,
            SomeTrafficToHostRule(targetIpAddress),
            SomeTrafficToOtherHostsRule(targetIpAddress),
        )
    }

    @Test
    @HasDependencyOnLocalAPI
    fun testEnsureNoLeaksToSpecificHostWhenSwitchingBetweenVariousVpnSettings() =
        runTest(timeout = 2.minutes) {
            app.launch()
            // Obfuscation and Post-Quantum are by default set to automatic. Explicitly set to off.
            app.applySettings(pq = QuantumResistantState.Off, obfuscationMode = ObfuscationMode.Off)

            on<ConnectPage> { clickSelectLocation() }

            on<SelectLocationPage> {
                clickLocationExpandButton(relayProvider.getDaitaRelay().country)
                clickLocationExpandButton(relayProvider.getDaitaRelay().city)
                clickLocationCell(relayProvider.getDaitaRelay().relay)
            }

            device.acceptVpnPermissionDialog()

            on<ConnectPage> { waitForConnectedLabel() }

            // Capture generated traffic to a specific host
            val targetIpAddress = InstrumentationRegistry.getArguments().getTrafficGeneratorHost()
            val targetPort = InstrumentationRegistry.getArguments().getTrafficGeneratorPort()
            val captureResult: PacketCaptureResult =
                PacketCapture().capturePackets {
                    TrafficGenerator(targetIpAddress, targetPort).generateTraffic(10.milliseconds) {
                        delay(
                            1000.milliseconds
                        ) // Give it some time for generating traffic in tunnel before changing
                        // settings

                        on<ConnectPage> { enableDAITAStory() }
                        on<ConnectPage> { setObfuscationStory(ObfuscationOption.Shadowsocks) }
                        on<ConnectPage> { waitForConnectedLabel() }

                        delay(
                            1000.milliseconds
                        ) // Give it some time for generating traffic in tunnel after enabling
                        // settings
                    }
                }

            val capturedStreams = captureResult.streams
            val capturedPcap = captureResult.pcap
            val timestamp = System.currentTimeMillis()
            Attachment.saveAttachment(
                "capture-${javaClass.enclosingMethod}-$timestamp.pcap",
                capturedPcap,
            )

            NetworkTrafficChecker.checkTrafficStreamsAgainstRules(
                capturedStreams,
                NoTrafficToHostRule(targetIpAddress),
            )
        }
}
