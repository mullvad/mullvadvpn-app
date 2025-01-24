package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_AUTH
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule : BeforeEachCallback {
    private val client =
        SimpleMullvadHttpClient(InstrumentationRegistry.getInstrumentation().targetContext)
    private val partnerAuth: String? =
        InstrumentationRegistry.getArguments().getString(PARTNER_AUTH, null)
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext) {
        InstrumentationRegistry.getArguments().also { bundle ->
            if (partnerAuth != null) {
                validAccountNumber = client.createAccountUsingPartnerApi(partnerAuth)
                client.addTimeToAccountUsingPartnerAuth(
                    accountNumber = validAccountNumber,
                    daysToAdd = 1,
                    partnerAuth = partnerAuth,
                )
            } else {
                validAccountNumber =
                    bundle.getRequiredArgument(VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
                client.removeAllDevices(validAccountNumber)
            }

            invalidAccountNumber =
                bundle.getRequiredArgument(INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
        }
    }
}
