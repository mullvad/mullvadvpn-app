package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.e2e.api.partner.PartnerApi
import net.mullvad.mullvadvpn.test.e2e.constant.INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.constant.PARTNER_AUTH
import net.mullvad.mullvadvpn.test.e2e.constant.VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY
import net.mullvad.mullvadvpn.test.e2e.extension.getRequiredArgument

object AccountProvider {
    private val mullvadClient = MullvadApi()
    private val partnerAuth: String? =
        InstrumentationRegistry.getArguments().getString(PARTNER_AUTH, null)
    private val partnerClient: PartnerApi by lazy { PartnerApi(partnerAuth!!) }

    suspend fun getValidAccountNumber() =
        // If partner auth is provided, create a new account using the partner API. Otherwise we
        // expect and account with time to be provided.
        if (partnerAuth != null) {
            val accountNumber = partnerClient.createAccount()
            partnerClient.addTime(accountNumber = accountNumber, daysToAdd = 1)
            accountNumber
        } else {
            val validAccountNumber =
                InstrumentationRegistry.getArguments()
                    .getRequiredArgument(VALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
            mullvadClient.removeAllDevices(validAccountNumber)
            validAccountNumber
        }

    fun getInvalidAccountNumber() =
        InstrumentationRegistry.getArguments()
            .getRequiredArgument(INVALID_TEST_ACCOUNT_NUMBER_ARGUMENT_KEY)
}
