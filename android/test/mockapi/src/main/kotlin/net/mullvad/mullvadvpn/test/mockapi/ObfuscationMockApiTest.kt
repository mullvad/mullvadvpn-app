package net.mullvad.mullvadvpn.test.mockapi

import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.disableIPv6Story
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ObfuscationMockApiTest : MockApiTest() {
    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    private val validAccountNumber = "1234123412341234"

    @BeforeEach
    fun setupDispatcher() {
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
    }

    @Test
    fun checkThatAllObfuscationsAreUsed() {
        app.launchAndLogIn(validAccountNumber)

        // Disable IPv6 so we do not test IPv6 with the same obfuscation method
        on<ConnectPage> { disableIPv6Story() }

        on<ConnectPage> {
            clickConnect()
            device.acceptVpnPermissionDialog()

            // Wait for obfuscation methods in order
            waitForConnectingLabel()
            obfuscationNames.forEach { waitForFeatureIndicator(it) }
        }
    }

    companion object {
        // List of obfuscation methods in the order the relay selector uses them
        // If the order in the relay selector changes this should be updated
        private val obfuscationNames = listOf("Shadowsocks", "QUIC", "UDP-over-TCP", "LWO")
    }
}
