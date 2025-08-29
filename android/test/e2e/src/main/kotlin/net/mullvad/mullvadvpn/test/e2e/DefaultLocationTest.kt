package net.mullvad.mullvadvpn.test.e2e

import java.io.File
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.json.JSONObject
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class DefaultLocationTest : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule()

    @Test
    fun testUpdateDefaultLocationFlag() {
        app.launchAndEnsureOnLoginPage()

        // The update_default_location flag should be set to true when first starting the app
        assert(readUpdateDefaultLocationKeyFromSettings())

        on<LoginPage> {
            enterAccountNumber(accountTestRule.validAccountNumber)
            clickLoginButton()
        }

        on<ConnectPage>()

        // After we have logged in the daemon will have set the new default location so the
        // flag should be false.
        assertFalse(readUpdateDefaultLocationKeyFromSettings())
    }

    private fun readUpdateDefaultLocationKeyFromSettings(): Boolean {
        val settings = File(targetApplication.filesDir, "settings.json")
        if (!settings.isFile()) error("settings.json does not exist")

        val text = settings.readText()
        val json = JSONObject(text)
        return json.getBoolean("update_default_location")
    }
}
