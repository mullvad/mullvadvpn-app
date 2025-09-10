package net.mullvad.mullvadvpn.test.baselineprofile

import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * This generates a baseline profile for the Mullvad VPN app. Run this from gradle with: ./gradlew
 * generatePlayProdReleaseBaselineProfile
 *
 * This should be done from time to time to keep the profile up to date with the app.
 *
 * NOTE: API 33+ or rooted API 28+ is required.
 */
@RunWith(AndroidJUnit4::class)
@LargeTest
class BaselineProfileGenerator {

    @get:Rule val rule = BaselineProfileRule()

    @Test
    fun generate() {
        rule.collect(
            packageName =
                InstrumentationRegistry.getArguments().getString("targetAppId")
                    ?: error("targetAppId not passed as instrumentation runner arg"),

            // See:
            // https://d.android.com/topic/performance/baselineprofiles/dex-layout-optimizations
            includeInStartupProfile = true,
        ) {
            pressHome()
            startActivityAndWait()
            device.acceptPrivacy()
        }
    }

    // This should use PrivacyPage from common but it is currently not possible to access the files
    // in that module from here. A fix for this is tracked in: DROID-2165
    private fun UiDevice.acceptPrivacy() {
        val agreeSelector = By.text("Agree and continue")
        findObject(agreeSelector)?.click()
    }
}
