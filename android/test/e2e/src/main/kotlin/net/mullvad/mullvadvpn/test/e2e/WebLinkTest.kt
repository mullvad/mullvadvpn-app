package net.mullvad.mullvadvpn.test.e2e

import net.mullvad.mullvadvpn.test.common.annotation.SkipForFlavors
import net.mullvad.mullvadvpn.test.common.page.MullvadWebsite
import net.mullvad.mullvadvpn.test.common.page.SettingsPage
import net.mullvad.mullvadvpn.test.common.page.TopBar
import net.mullvad.mullvadvpn.test.common.page.on
import org.junit.jupiter.api.Test

class WebLinkTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {
    @Test
    @SkipForFlavors(currentFlavor = BuildConfig.FLAVOR_billing, "play")
    fun testOpenFaqFromApp() {
        app.launchAndEnsureOnLoginPage()

        on<TopBar> { clickSettings() }

        on<SettingsPage> { clickFaqAndGuides() }

        on<MullvadWebsite> {}
    }
}
