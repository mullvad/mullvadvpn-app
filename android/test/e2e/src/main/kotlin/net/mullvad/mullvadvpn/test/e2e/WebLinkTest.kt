package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.constant.WEB_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import org.junit.jupiter.api.Test

class WebLinkTest : EndToEndTest() {
    @Test
    fun testOpenFaqFromApp() {
        // Given
        app.launch()

        // When
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.findObjectWithTimeout(By.text("Login"))
        app.clickSettingsCog()
        app.clickListItemByText("FAQs & Guides")

        // Then
        device.findObjectWithTimeout(
            selector = By.text("Mullvad help center"),
            timeout = WEB_TIMEOUT
        )
    }
}
