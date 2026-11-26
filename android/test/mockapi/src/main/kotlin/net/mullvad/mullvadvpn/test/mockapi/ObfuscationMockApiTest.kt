package net.mullvad.mullvadvpn.test.mockapi

import java.time.ZonedDateTime
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
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
        apiRouter.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
    }

    @Test
    fun checkThatAllObfuscationsAreUsed() = runTest {
        app.launchAndLogIn(validAccountNumber)

        app.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()

        // Disable IPv6 so we do not test IPv6 with the same obfuscation method
        app.applySettings(deviceIpVersion = Constraint.Only(IpVersion.IPV4))

        on<ConnectPage> {
            clickConnect()
            device.acceptVpnPermissionDialog()
        }

        // Collect tunnel states until all obfuscation methods have been seen
        val remaining = obfuscationIndicators.toMutableSet()
        app.tunnelStateFlow()
            // ConnectionState.featureIndicators() will filter out multiple obfuscation methods used
            // at the same time.
            .mapNotNull { it.allFeatureIndicators() }
            .first {
                remaining.removeAll(it.toSet())
                remaining.isEmpty()
            }
    }

    private fun TunnelState.allFeatureIndicators() =
        when (this) {
            is TunnelState.Connecting -> featureIndicators
            is TunnelState.Connected -> featureIndicators
            else -> null
        }

    companion object {
        // All currently reported obfuscation methods
        private val obfuscationIndicators =
            listOf(
                FeatureIndicator.LWO,
                FeatureIndicator.QUIC,
                FeatureIndicator.SHADOWSOCKS,
                FeatureIndicator.UDP_2_TCP,
            )
    }
}
