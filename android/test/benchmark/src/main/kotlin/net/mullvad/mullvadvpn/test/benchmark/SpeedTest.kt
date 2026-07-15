package net.mullvad.mullvadvpn.test.benchmark

import co.touchlab.kermit.Logger
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.test.benchmark.rule.AccountTestRule
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.misc.Attachment
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class SpeedTest : BenchmarkTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    @Test
    @Disabled(
        "Testing iperf3 without a VPN connection to the stagemole relay is currently not possible"
    )
    fun noVpn() {
        val result = runIperf3(context, targetContext)

        if (result.error != null) {
            Logger.e("Error: ${result.error}")
            error("Received an error from iperf3 ${result.error}")
        }

        val timestamp = System.currentTimeMillis()
        Attachment.saveAttachment(
            "benchmark-${javaClass.enclosingMethod}-$timestamp.json",
            result.toString(),
        )
    }

    @Test
    fun testNoObfuscation() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        app.applySettings(
            obfuscationMode = ObfuscationMode.Off,
            location = IPERF_RELAY_ID,
        )

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context, targetContext)

        if (result.error != null) {
            Logger.e("Error: ${result.error}")
            error("Received an error from iperf3")
        }

        val prefix = "benchmark-no-obfuscation-${System.currentTimeMillis()}"
        Attachment.saveAttachment(
            "$prefix-summary.txt",
            result.summarize(),
        )
        Attachment.saveAttachment(
            "$prefix.json",
            Json.encodeToString(result),
        )
    }

    @Test
    fun testUdpOverTcpObfuscation() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        app.applySettings(
            obfuscationMode = ObfuscationMode.Udp2Tcp,
            location = IPERF_RELAY_ID,
        )

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context, targetContext)

        if (result.error != null) {
            Logger.e("Error: ${result.error}")
            error("Received an error from iperf3")
        }

        val prefix = "benchmark-udp-over-tcp-obfuscation-${System.currentTimeMillis()}"
        Attachment.saveAttachment(
            "$prefix-summary.txt",
            result.summarize(),
        )
        Attachment.saveAttachment(
            "$prefix.json",
            Json.encodeToString(result),
        )
    }

    @Test
    fun testShadowsocks() = runTest {
        // Given
        app.launchAndLogIn(accountTestRule.validAccountNumber)

        app.applySettings(
            obfuscationMode = ObfuscationMode.Shadowsocks,
            location = IPERF_RELAY_ID,
        )

        on<ConnectPage> { clickConnect() }

        device.acceptVpnPermissionDialog()

        on<ConnectPage> { waitForConnectedLabel() }

        val result = runIperf3(context, targetContext)

        if (result.error != null) {
            Logger.e("Error: ${result.error}")
            error("Received an error from iperf3")
        }

        val prefix = "benchmark-shadowsocks-obfuscation-${System.currentTimeMillis()}"
        Attachment.saveAttachment(
            "$prefix-summary.txt",
            result.summarize(),
        )
        Attachment.saveAttachment(
            "$prefix.json",
            Json.encodeToString(result),
        )
    }

    companion object {
        private val IPERF_RELAY_ID =
            GeoLocationId.Hostname(
                city =
                    GeoLocationId.City(
                        country = GeoLocationId.Country(code = "se"),
                        code = "got",
                    ),
                code = "se-got-wg-121",
            )
    }
}
