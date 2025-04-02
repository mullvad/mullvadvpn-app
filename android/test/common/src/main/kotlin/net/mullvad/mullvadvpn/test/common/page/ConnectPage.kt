package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.pressBackTwice

class ConnectPage internal constructor() : Page() {
    private val disconnectSelector = By.text("Disconnect")
    private val cancelSelector = By.text("Cancel")
    private val connectedSelector = By.text("CONNECTED")
    private val connectingSelector = By.text("CONNECTING...")
    private val disconnectedSelector = By.text("DISCONNECTED")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG))
    }

    fun clickSettings() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
    }

    fun clickAccount() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_ACCOUNT_BUTTON)).click()
    }

    fun clickSelectLocation() {
        uiDevice.findObjectWithTimeout(By.res(SELECT_LOCATION_BUTTON_TEST_TAG)).click()
    }

    fun clickConnect() {
        uiDevice.findObjectWithTimeout(By.res(CONNECT_BUTTON_TEST_TAG)).click()
    }

    fun clickDisconnect() {
        uiDevice.findObjectWithTimeout(disconnectSelector).click()
    }

    fun clickCancel() {
        uiDevice.findObjectWithTimeout(cancelSelector).click()
    }

    fun waitForConnectedLabel(timeout: Long = VERY_LONG_TIMEOUT) {
        uiDevice.findObjectWithTimeout(connectedSelector, timeout)
    }

    fun waitForDisconnectedLabel(timeout: Long = VERY_LONG_TIMEOUT) {
        uiDevice.findObjectWithTimeout(disconnectedSelector, timeout)
    }

    fun waitForConnectingLabel() {
        uiDevice.findObjectWithTimeout(connectingSelector)
    }

    /**
     * Extracts the in IPv4 address from the connection card. It is a prerequisite that the
     * connection card is in collapsed state.
     */
    fun extractInIpv4Address(): String {
        uiDevice.findObjectWithTimeout(By.res("connect_card_header_test_tag")).click()
        val inString =
            uiDevice
                .findObjectWithTimeout(
                    By.res("location_info_connection_in_test_tag"),
                    VERY_LONG_TIMEOUT,
                )
                .text

        val extractedIpAddress = inString.split(" ")[0].split(":")[0]
        return extractedIpAddress
    }

    /**
     * Extracts the out IPv4 address from the connection card. It is a prerequisite that the
     * connection card is in collapsed state.
     */
    fun extractOutIpv4Address(): String {
        uiDevice.findObjectWithTimeout(By.res("connect_card_header_test_tag")).click()
        return uiDevice
            .findObjectWithTimeout(
                // Text exist and contains IP address
                By.res("location_info_connection_out_test_tag").textContains("."),
                VERY_LONG_TIMEOUT,
            )
            .text
    }

    fun disableObfuscation() {
        clickSettings()
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationOffCell()
            clickWireGuardObfuscationOffCell()
        }
        uiDevice.pressBackTwice()
    }

    fun disablePostQuantum() {
        clickSettings()
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilPostQuantumOffCell()
            clickPostQuantumOffCell()
        }
        uiDevice.pressBackTwice()
    }

    fun enableShadowsocks() {
        clickSettings()
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> {
            scrollUntilWireGuardObfuscationShadowsocksCell()
            clickWireGuardObfuscationShadowsocksCell()
        }
        uiDevice.pressBackTwice()
    }

    fun enableDAITA() {
        clickSettings()
        on<SettingsPage> { clickDaita() }
        on<DaitaSettingsPage> { clickEnableSwitch() }
        uiDevice.pressBackTwice()
    }

    fun enableLocalNetworkSharing() {
        clickSettings()
        on<SettingsPage> { clickVpnSettings() }
        on<VpnSettingsPage> { clickLocalNetworkSharingSwitch() }
        uiDevice.pressBackTwice()
    }

    companion object {
        const val TOP_BAR_ACCOUNT_BUTTON = "top_bar_account_button"
        const val TOP_BAR_SETTINGS_BUTTON = "top_bar_settings_button"
        const val CONNECT_CARD_HEADER_TEST_TAG = "connect_card_header_test_tag"
        const val SELECT_LOCATION_BUTTON_TEST_TAG = "select_location_button_test_tag"
        const val CONNECT_BUTTON_TEST_TAG = "connect_button_test_tag"
    }
}
