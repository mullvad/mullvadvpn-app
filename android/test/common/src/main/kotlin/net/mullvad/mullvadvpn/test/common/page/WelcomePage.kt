package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObjectNotFoundException
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_ACCOUNT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class WelcomePage internal constructor() : Page() {
    private val welcomeSelector = By.text("Congrats!")

    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(welcomeSelector)
    }

    fun clickAccount() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_ACCOUNT_BUTTON_TEST_TAG)).click()
    }
}

fun UiDevice.dismissStorePasswordPromptIfShown() {
    try {
        val selector = By.textContains("password")
        findObjectWithTimeout(selector)
        pressBack()
    } catch (_: UiObjectNotFoundException) {
        Logger.d("No password manager prompt found")
    }
}
