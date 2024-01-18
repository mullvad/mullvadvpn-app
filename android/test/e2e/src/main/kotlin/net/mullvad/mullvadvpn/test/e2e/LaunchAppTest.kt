package net.mullvad.mullvadvpn.test.e2e

import org.junit.jupiter.api.Test

class LaunchAppTest : EndToEndTest(BuildConfig.FLAVOR_infrastructure) {
    @Test
    fun testLaunchApp() {
        app.launch()
    }
}
