package net.mullvad.mullvadvpn.test.e2e

import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.MullvadWebsite
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.on
import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class WebLinkTest : EndToEndTest() {
    @Test
    @Disabled("Disabled due to broken in-browser text detection (DROID-2009)")
    fun testOpenFaqFromApp() {
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> { clickSettings() }

        on<SettingsPage> { clickFaqAndGuides() }

        on<MullvadWebsite>()
    }
}
