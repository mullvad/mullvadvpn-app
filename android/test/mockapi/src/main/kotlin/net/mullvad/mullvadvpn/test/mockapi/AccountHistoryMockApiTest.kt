package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.AccountPage
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Test

class AccountHistoryMockApiTest : MockApiTest() {

    @Test
    fun testShowAccountHistory() {
        // Arrange
        val validAccountNumber = "1234123412341234"
        apiRouter.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }

        app.launchAndLogIn(validAccountNumber)

        on<ConnectPage> { clickAccount() }

        on<AccountPage> { clickLogOut() }

        val expectedResult = "1234 1234 1234 1234"
        on<LoginPage> {
            toggleRevealInput()
            assertHasAccountHistory(expectedResult)
            device.findObjectWithTimeout(By.text(expectedResult)).click()
        }

        on<ConnectPage>()
    }
}
