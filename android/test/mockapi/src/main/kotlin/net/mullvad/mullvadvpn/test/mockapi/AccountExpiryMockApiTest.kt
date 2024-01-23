package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import net.mullvad.mullvadvpn.test.mockapi.util.currentUtcTimeWithOffsetZero
import net.mullvad.mullvadvpn.util.toExpiryDateString
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class AccountExpiryMockApiTest : MockApiTest() {

    @Test
    fun testAccountExpiryDateUpdated() {
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

        // Add one month to the account expiry
        val newAccountExpiry = oldAccountExpiry.plusMonths(1)
        apiDispatcher.accountExpiry = newAccountExpiry

        // Go to account page to update the account expiry
        app.clickAccountCog()

        app.ensureAccountScreen()
        device.findObjectWithTimeout(By.text(newAccountExpiry.toExpiryDateString()))
    }

    @Test
    @Disabled(
        "Disabled since we have a bug in the app that makes it unstable. " +
            "We can restore it after the bug has been fixed"
    )
    fun testAccountTimeExpiredWhileUsingTheAppShouldShowOutOfTimeScreen() {
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

        // Set account time as expired
        val newAccountExpiry = oldAccountExpiry.minusMonths(2)
        apiDispatcher.accountExpiry = newAccountExpiry

        // Go to account page to update the account expiry
        app.clickAccountCog()
        app.ensureAccountScreen()

        // Go back to the main screen
        device.pressBack()

        // Assert that we show the out of time screen
        app.ensureOutOfTime()
    }
}
