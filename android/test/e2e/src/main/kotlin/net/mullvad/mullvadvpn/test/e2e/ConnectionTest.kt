package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.ConnCheckState
import net.mullvad.mullvadvpn.test.e2e.misc.SimpleMullvadHttpClient
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ConnectionTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    @Test
    fun testConnect() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.findObjectWithTimeout(By.text("OK")).click()

        // Then
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"))
    }

    @Test
    @Disabled("Disabled since the connection check isn't reliable in the stagemole infrastructure.")
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.findObjectWithTimeout(By.text("OK")).click()
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"))
        val expected = ConnCheckState(true, app.extractIpAddress())

        // Then
        val result = SimpleMullvadHttpClient(targetContext).runConnectionCheck()
        assertEquals(expected, result)
    }
}
