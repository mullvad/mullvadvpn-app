package net.mullvad.mullvadvpn.test.e2e

import net.mullvad.mullvadvpn.test.common.annotation.SkipForFlavors
import net.mullvad.mullvadvpn.test.common.extension.waitForStableInActiveWindowSafe
import net.mullvad.mullvadvpn.test.common.page.AddTimeBottomSheet
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.GooglePlayPaymentPage
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.OutOfTimePage
import net.mullvad.mullvadvpn.test.common.page.WelcomePage
import net.mullvad.mullvadvpn.test.common.page.dismissStorePasswordPromptIfShown
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class PaymentTest : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule(withTime = false)

    @Test
    @SkipForFlavors(currentFlavor = BuildConfig.FLAVOR_billing, "oss")
    fun testInAppPurchaseForOutOfTime() {
        val validTestAccountNumber = accountTestRule.validAccountNumber

        app.launchAndEnsureOnLoginPage()

        on<LoginPage> {
            enterAccountNumber(validTestAccountNumber)
            clickLoginButton()
        }

        on<OutOfTimePage> { clickAddTime() }

        on<AddTimeBottomSheet> { click30days() }

        on<GooglePlayPaymentPage> { clickBuy() }

        // Just wait some extra time to ensure the purchase is processed
        device.waitForStableInActiveWindowSafe()

        // Ensure that after successful purchase, the app navigates to the Connect page
        on<ConnectPage>()
    }

    @Test
    @SkipForFlavors(currentFlavor = BuildConfig.FLAVOR_billing, "oss")
    fun testInAppPurchaseForWelcome() {
        app.launchAndEnsureOnLoginPage()

        on<LoginPage> { clickCreateAccount() }

        device.dismissStorePasswordPromptIfShown()

        on<WelcomePage> { clickAddTime() }

        on<AddTimeBottomSheet> { click30days() }

        on<GooglePlayPaymentPage> { clickBuy() }

        // Just wait some extra time to ensure the purchase is processed
        device.waitForStableInActiveWindowSafe()

        // Ensure that after successful purchase, the app navigates to the Connect page
        on<ConnectPage>()
    }
}
