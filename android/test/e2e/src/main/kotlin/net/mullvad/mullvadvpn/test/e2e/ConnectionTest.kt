package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import junit.framework.Assert.assertEquals
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import net.mullvad.mullvadvpn.test.e2e.misc.ConnCheckState
import net.mullvad.mullvadvpn.test.e2e.misc.SimpleMullvadHttpClient
import org.junit.Rule
import org.junit.Test

class ConnectionTest : EndToEndTest() {

    @Rule
    @JvmField
    val cleanupAccountTestRule = CleanupAccountTestRule()

    @Rule
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    @Test
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(validTestAccountToken)

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
