package net.mullvad.mullvadvpn.test.baselineprofile

import android.app.Application
import androidx.benchmark.macro.junit4.BaselineProfileRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiObjectNotFoundException
import androidx.test.uiautomator.waitForStableInActiveWindow
import net.mullvad.mullvadvpn.test.common.interactor.AppInteractor
import net.mullvad.mullvadvpn.test.common.page.AccountPage
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.PrivacyPage
import net.mullvad.mullvadvpn.test.common.page.WelcomePage
import net.mullvad.mullvadvpn.test.common.page.dismissStorePasswordPromptIfShown
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
            // We will rate limited if we create more than 3 new accounts
            maxIterations = 3,
        ) {
            pressHome()
            startActivityAndWait()

            val targetApplication =
                InstrumentationRegistry.getInstrumentation().targetContext.applicationContext
                    as Application

            val app = AppInteractor(device, targetApplication)

            ignoreNotFound { on<PrivacyPage> { clickAgreeOnPrivacyDisclaimer() } }
            ignoreNotFound { app.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove() }
            on<LoginPage> { clickCreateAccount() }
            device.dismissStorePasswordPromptIfShown()
            on<WelcomePage> { clickAccount() }
            on<AccountPage> { clickLogOut() }
            on<LoginPage> {
                deleteAccountHistory()
                enterAccountNumber(getValidAccountNumber())
                device.waitForStableInActiveWindow()
                clickLoginButton()
            }
            // Clean up for next run
            on<ConnectPage> {
                clickAccount()
            }
            on<AccountPage> {
                clickLogOut()
            }
            on<LoginPage> {
                deleteAccountHistory()
            }
        }
    }

    private fun ignoreNotFound(block: () -> Unit) {
        try {
            block()
        } catch (_: UiObjectNotFoundException) {}
    }

    private fun getValidAccountNumber() =
        InstrumentationRegistry.getArguments()
            .getString("mullvad.test.baseline.accountNumber.valid")
            ?: error("Requires a valid prod account number")
}
