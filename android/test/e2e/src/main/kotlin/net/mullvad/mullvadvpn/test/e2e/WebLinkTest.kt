package net.mullvad.mullvadvpn.test.e2e

import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.MullvadWebsite
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.on
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class WebLinkTest : EndToEndTest() {
    @Test
    @Disabled(
        "This test currently does not work with an instrumentor test app " +
            "since it is unable to detect any text in the browser" +
        "This is tracked in DROID-2009"
    )
    fun testOpenFaqFromApp() {
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> { clickSettings() }

        on<SettingsPage> { clickFaqAndGuides() }

        on<MullvadWebsite>()
    }
}
