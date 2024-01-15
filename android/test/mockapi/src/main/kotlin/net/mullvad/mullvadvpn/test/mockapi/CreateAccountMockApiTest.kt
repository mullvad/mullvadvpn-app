package net.mullvad.mullvadvpn.test.mockapi

import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import org.junit.jupiter.api.Test

class CreateAccountMockApiTest : MockApiTest() {
    @Test
    fun testCreateAccountSuccessful() {
        // Arrange
        val createdAccountToken = "1234123412341234"
        apiDispatcher.apply { expectedAccountToken = createdAccountToken }
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptCreateAccount()

        // Assert
        app.ensureAccountCreated(createdAccountToken.groupWithSpaces())
    }

    @Test
    fun testCreateAccountFailed() {
        // Arrange
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptCreateAccount()

        // Assert
        app.ensureAccountCreationFailed()
    }
}
