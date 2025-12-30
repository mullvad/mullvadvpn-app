package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.lib.ui.tag.TOP_BAR_ACCOUNT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
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
        wait(Until.hasObject(selector), DEFAULT_TIMEOUT)
        pressBack()
    } catch (_: IllegalArgumentException) {
        // This is OK since it means the password prompt wasn't shown.
    }
}
