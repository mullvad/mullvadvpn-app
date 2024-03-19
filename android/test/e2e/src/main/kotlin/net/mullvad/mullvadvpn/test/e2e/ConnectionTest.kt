package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.CONNECTION_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickOkOnVpnPermissionPrompt
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
        device.clickOkOnVpnPermissionPrompt()

        // Then
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"), CONNECTION_TIMEOUT)
    }

    @Test
    fun givenValidAccountAndPQOnShouldConnectQuantumSecure() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)
        // Go to settings and Set PQ to true
        app.clickSettingsCog()
        app.clickListItemByText("VPN settings")
        app.scrollToListItemAndClick("lazy_list_test_tag", "lazy_list_quantum_item_on_test_tag")
        // Go back to the connect screen
        device.pressBack()
        device.pressBack()

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.clickOkOnVpnPermissionPrompt()

        // Then
        device.findObjectWithTimeout(By.text("QUANTUM SECURE CONNECTION"), CONNECTION_TIMEOUT)
    }

    @Test
    @Disabled("Disabled since the connection check isn't reliable in the stagemole infrastructure.")
    fun testConnectAndVerifyWithConnectionCheck() {
        // Given
        app.launchAndEnsureLoggedIn(accountTestRule.validAccountNumber)

        // When
        device.findObjectWithTimeout(By.text("Secure my connection")).click()
        device.clickOkOnVpnPermissionPrompt()
        device.findObjectWithTimeout(By.text("SECURE CONNECTION"))
        val expected = ConnCheckState(true, app.extractIpAddress())

        // Then
        val result = SimpleMullvadHttpClient(targetContext).runConnectionCheck()
        assertEquals(expected, result)
    }
}
