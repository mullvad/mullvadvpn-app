package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.CONNECT_CARD_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_INFO_CONNECTION_IN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_INFO_CONNECTION_OUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_ACCOUNT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_SETTINGS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

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
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON_TEST_TAG)).click()
    }

    fun clickAccount() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_ACCOUNT_BUTTON_TEST_TAG)).click()
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
        uiDevice.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG)).click()
        val inString =
            uiDevice
                .findObjectWithTimeout(
                    By.res(LOCATION_INFO_CONNECTION_IN_TEST_TAG),
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
        uiDevice.findObjectWithTimeout(By.res(CONNECT_CARD_HEADER_TEST_TAG)).click()
        return uiDevice
            .findObjectWithTimeout(
                // Text exist and contains IP address
                By.res(LOCATION_INFO_CONNECTION_OUT_TEST_TAG).textContains("."),
                VERY_LONG_TIMEOUT,
            )
            .text
    }
}
