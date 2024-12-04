package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class TopBar internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_TEST_TAG))
    }

    fun clickSettings() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_SETTINGS_BUTTON)).click()
    }

    fun clickAccount() {
        uiDevice.findObjectWithTimeout(By.res(TOP_BAR_ACCOUNT_BUTTON)).click()
    }

    companion object {
        const val TOP_BAR_TEST_TAG = "top_bar_test_tag"
        const val TOP_BAR_ACCOUNT_BUTTON = "top_bar_account_button"
        const val TOP_BAR_SETTINGS_BUTTON = "top_bar_settings_button"
    }
}
