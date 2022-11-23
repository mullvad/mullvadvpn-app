package net.mullvad.mullvadvpn.test.e2e

import androidx.test.runner.AndroidJUnit4
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LaunchAppTest : EndToEndTest() {
    @Test
    fun testLaunchApp() {
        app.launch()
    }
}
