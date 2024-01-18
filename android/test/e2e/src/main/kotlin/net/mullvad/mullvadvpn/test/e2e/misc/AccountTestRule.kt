package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_AUTH
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule : BeforeEachCallback {

    private val partnerAccount: String?
    private val client =
        SimpleMullvadHttpClient(InstrumentationRegistry.getInstrumentation().targetContext)

    val validAccountNumber: String
    val invalidAccountNumber: String

    init {
        InstrumentationRegistry.getArguments().also { bundle ->
            partnerAccount = bundle.getString(PARTNER_AUTH)

            if (partnerAccount != null) {
                validAccountNumber = client.createAccount()
                client.addTimeToAccountUsingPartnerAuth(
                    accountNumber = validAccountNumber,
                    daysToAdd = 1,
                    partnerAuth = partnerAccount
                )
            } else {
                validAccountNumber =
                    bundle.getRequiredArgument(VALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
                client.removeAllDevices(validAccountNumber)
            }

            invalidAccountNumber =
                bundle.getRequiredArgument(INVALID_TEST_ACCOUNT_TOKEN_ARGUMENT_KEY)
        }
    }

    override fun beforeEach(context: ExtensionContext) {}
}
