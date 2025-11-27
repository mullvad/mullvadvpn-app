package net.mullvad.mullvadvpn.test.mockapi

import java.time.ZonedDateTime
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.misc.RelayProvider
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.enableServerIpOverrideStory
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class ServerIpOverridesMockApiTest : MockApiTest() {

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    // We are using a static relay list in mock api tests that mirrors the production relay list, so
    // we should always check for production relays.
    private val relayProvider = RelayProvider("oss")
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
    fun testAttemptToConnectUsingServerIpOverride() = runTest {
        // Arrange
        app.launchAndLogIn(validAccountNumber)

        // Enable server ip override
        val mockServerIp = "12.12.12.12"
        val relay = relayProvider.getOverrideRelay()
        on<ConnectPage> { enableServerIpOverrideStory(relay.relay, mockServerIp) }

        // Select the relay which has an overriden ip
        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickLocationExpandButton(relay.country)
            clickLocationExpandButton(relay.city)
            clickLocationCell(relay.relay)
        }

        device.acceptVpnPermissionDialog()

        var inIpv4Address = ""

        on<ConnectPage> { inIpv4Address = extractInIpv4Address() }

        // Verify connection
        assertEquals(mockServerIp, inIpv4Address)
    }
}
