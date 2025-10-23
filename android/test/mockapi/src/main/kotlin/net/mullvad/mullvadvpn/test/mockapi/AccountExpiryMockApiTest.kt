package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.AccountPage
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.OutOfTimePage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.util.toExpiryDateString
import org.junit.jupiter.api.Test

class AccountExpiryMockApiTest : MockApiTest() {

    @Test
    fun testAccountExpiryDateUpdated() {
        // Arrange
        val (validAccountNumber, oldAccountExpiry) = configureAccount()

        // Act
        app.launchAndLogIn(validAccountNumber)

        // Add one month to the account expiry
        val newAccountExpiry = oldAccountExpiry.plusMonths(1)
        apiDispatcher.accountExpiry = newAccountExpiry

        on<ConnectPage> { clickAccount() }

        on<AccountPage> {
            // Assert that the account expiry date is updated
            device.findObjectWithTimeout(By.text(newAccountExpiry.toExpiryDateString()))
        }
    }

    @Test
    fun testAccountTimeExpiredWhileUsingTheAppShouldShowOutOfTimeScreen() {
        // Arrange
        val (validAccountNumber, oldAccountExpiry) = configureAccount()

        // Act
        app.launchAndLogIn(validAccountNumber)

        // Wait for us to be on connect page before changing expiry
        on<ConnectPage>()

        // Set account time as expired
        val newAccountExpiry = oldAccountExpiry.minusMonths(2)
        apiDispatcher.accountExpiry = newAccountExpiry

        on<ConnectPage> { clickAccount() }

        // Go to account page to update the account expiry
        on<AccountPage>()

        // Go back to the main screen
        device.pressBack()

        // Assert that we show the out of time screen
        on<OutOfTimePage>()
    }

    @Test
    fun testAccountTimeExpiryNotificationIsShown() {
        // Arrange
        val (validAccountNumber, _) = configureAccount()

        // Act
        app.launchAndLogIn(validAccountNumber)

        // Wait for us to be on connect page before changing expiry
        on<ConnectPage>()

        // Set account time as expired
        val newAccountExpiry = ZonedDateTime.now().plusHours(3 * 24).plusSeconds(5)
        apiDispatcher.accountExpiry = newAccountExpiry

        on<ConnectPage> { clickAccount() }

        // Go to account page to update the account expiry
        on<AccountPage>()

        // Go back to the main screen
        device.openNotification()

        // Make sure the notification is shown
        device.findObjectWithTimeout(
            By.text("Account credit expires in 2 days"),
            timeout = VERY_LONG_TIMEOUT,
        )
    }

    private fun configureAccount(): Pair<String, ZonedDateTime> {
        val validAccountNumber = "1234123412341234"
        val oldAccountExpiry = ZonedDateTime.now().plusMonths(1)
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = oldAccountExpiry
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
        return Pair(validAccountNumber, oldAccountExpiry)
    }
}
