package net.mullvad.mullvadvpn.test.e2e

import androidx.test.uiautomator.By
import net.mullvad.mullvadvpn.lib.ui.tag.CONNECT_CARD_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.test.common.annotation.SkipForFlavors
import net.mullvad.mullvadvpn.test.common.constant.VERY_LONG_TIMEOUT
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.page.AddTimeBottomSheet
import net.mullvad.mullvadvpn.test.common.page.LoginPage
import net.mullvad.mullvadvpn.test.common.page.OutOfTimePage
import net.mullvad.mullvadvpn.test.common.page.buyGooglePlayTime
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.e2e.annotations.RequiresGoogleBillingAccount
import net.mullvad.mullvadvpn.test.e2e.annotations.RequiresPartnerAuth
import net.mullvad.mullvadvpn.test.e2e.misc.AccountTestRule
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

class PaymentTest : EndToEndTest() {

    @RegisterExtension @JvmField val accountTestRule = AccountTestRule(withTime = false)

    @Test
    @SkipForFlavors(currentFlavor = BuildConfig.FLAVOR_billing, "oss")
    @RequiresGoogleBillingAccount
    @RequiresPartnerAuth
    fun testInAppPurchaseForOutOfTime() {
        val validTestAccountNumber = accountTestRule.validAccountNumber

        app.launchAndEnsureOnLoginPage()

        on<LoginPage> {
            enterAccountNumber(validTestAccountNumber)
            clickLoginButton()
        }

        on<OutOfTimePage> { clickAddTime() }

        on<AddTimeBottomSheet> { click30days() }

        device.buyGooglePlayTime()

        // Assert we reach the Connect page after purchase
        device.findObjectWithTimeout(
            By.res(CONNECT_CARD_HEADER_TEST_TAG),
            timeout = VERY_LONG_TIMEOUT,
        )
    }
}
