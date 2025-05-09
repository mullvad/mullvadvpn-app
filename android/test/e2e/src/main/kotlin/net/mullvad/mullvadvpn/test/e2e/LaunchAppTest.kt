package net.mullvad.mullvadvpn.test.e2e

import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

class LaunchAppTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {

    @Disabled
    @Test
    fun testLaunchApp() {
        app.launch()
    }
}
