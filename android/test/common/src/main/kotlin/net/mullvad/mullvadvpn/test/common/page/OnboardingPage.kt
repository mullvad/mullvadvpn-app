package net.mullvad.mullvadvpn.test.common.page

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout

class OnboardingPage internal constructor() : Page() {
    override fun assertIsDisplayed() {
        uiDevice.findObjectWithTimeout(By.res(ONBOARDING_SCREEN_TEST_TAG))
    }
}
