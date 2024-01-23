package net.mullvad.mullvadvpn.test.mockapi

import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import org.junit.jupiter.api.Test

class AccountExpiryMockApiTest : MockApiTest() {

    @Test
    fun testOpenAccountPageOfAccountThatJustExpired() {
        // Arrange
        val validAccountToken = "1234123412341234"
        val oldAccountExpiry = currentUtcTimeWithOffsetZero().plusMonths(1)
        apiDispatcher.apply {
            expectedAccountToken = validAccountToken
            accountExpiry = oldAccountExpiry
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }

        // Act
        app.launch(endpoint)
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptLogin(validAccountToken)

        // Assert logged in
        app.ensureLoggedIn()

        // Set up account expiry to be in the past
        val newAccountExpiry = currentUtcTimeWithOffsetZero().minusDays(1)
        apiDispatcher.accountExpiry = newAccountExpiry

        // Go to account page to update the account expiry
        app.clickAccountCog()

        // Some times the out of time screen will show up, sometimes it won't
        try {
            app.ensureOutOfTime()
        } catch (e: IllegalArgumentException) {
            // Just go back to the connect page and try again
            device.pressBack()
            app.ensureOutOfTime()
        }
    }
}
