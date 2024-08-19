package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_AUTH
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

class AccountTestRule : BeforeEachCallback {
    private var partnerAccount: String? = null
    private val client =
        SimpleMullvadHttpClient(InstrumentationRegistry.getInstrumentation().targetContext)
    lateinit var validAccountNumber: String
    lateinit var invalidAccountNumber: String

    override fun beforeEach(context: ExtensionContext) {
        InstrumentationRegistry.getArguments().also { bundle ->
            partnerAccount = bundle.getString(PARTNER_AUTH)

            partnerAccount?.let { partnerAccount ->
                validAccountNumber = client.createAccount()
                client.addTimeToAccountUsingPartnerAuth(
                    accountNumber = validAccountNumber,
                    daysToAdd = 1,
                    partnerAuth = partnerAccount
                )
            }
                ?: run {
                    validAccountNumber =
                        bundle.getRequiredArgument(VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
                    client.removeAllDevices(validAccountNumber)
                }

            invalidAccountNumber =
                bundle.getRequiredArgument(INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
        }
    }
}
