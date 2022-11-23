package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import junit.framework.Assert.assertEquals
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.e2e.interactor.WebViewInteractor
import net.mullvad.mullvadvpn.test.e2e.misc.CleanupAccountTestRule
import org.junit.Rule
import org.junit.Test

class ConnectionTest : EndToEndTest() {

    @Rule
    @JvmField
    val cleanupAccountTestRule = CleanupAccountTestRule()

    @Test
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(validTestAccountToken)

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.findObjectWithTimeout(By.text("OK")).click()
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"))
        val expected = WebViewInteractor.ConnCheckState(true, app.extractIpAddress())

        // Then
        val result = web.launchAndExtractConnCheckState()
        assertEquals(expected, result)
    }
}
