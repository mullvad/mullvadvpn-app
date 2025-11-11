package net.mullvad.mullvadvpn.test.baselineprofile

import android.app.Application
import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiObjectNotFoundException
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.PrivacyPage
import net.mullvad.mullvadvpn.test.common.page.on
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

            val targetApplication =
                InstrumentationRegistry.getInstrumentation().targetContext.applicationContext
                    as Application

            val app = AppInteractor(device, targetApplication)

            ignoreNotFound { on<PrivacyPage> { clickAgreeOnPrivacyDisclaimer() } }
            ignoreNotFound { app.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove() }
            on<LoginPage>()
        }
    }

    fun ignoreNotFound(block: () -> Unit) {
        try {
            block()
        } catch (_: UiObjectNotFoundException) {}
    }
}
