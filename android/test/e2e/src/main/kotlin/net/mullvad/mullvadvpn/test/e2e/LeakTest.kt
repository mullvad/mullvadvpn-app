package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import co.touchlab.kermit.Logger
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCapture
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCaptureClient
import net.mullvad.mullvadvpn.test.e2e.misc.PacketCaptureSession
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class LeakTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    val packetCaptureClient = PacketCaptureClient()

    @Test
    fun testNegativeLeak() = runBlocking<Unit> {
        val packetCapture = PacketCapture()

        val session = packetCapture.startCapture()
        packetCaptureClient.sendStartCaptureRequest(session)
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.findObjectWithTimeout(By.text("OK")).click()

        // Then
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"), CONNECTION_TIMEOUT)

        device.findObjectWithTimeout(By.text("Disconnect")).click()
        Thread.sleep(2000)

        packetCaptureClient.sendStopCaptureRequest(session)
        val parsedObjects = packetCapture.stopCapture(session)
        //Logger.v("Parsed packet capture objects: $parsedObjects")
    }
}
