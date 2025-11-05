package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.e2e.api.partner.PartnerApi
import net.mullvad.mullvadvpn.test.e2e.constant.getInvalidAccountNumber
import net.mullvad.mullvadvpn.test.e2e.constant.getPartnerAuth
import net.mullvad.mullvadvpn.test.e2e.constant.getValidAccountNumber

object AccountProvider {
    private val mullvadClient = MullvadApi()
    private val partnerAuth: String? = InstrumentationRegistry.getArguments().getPartnerAuth()
    private val partnerClient: PartnerApi by lazy { PartnerApi(partnerAuth!!) }

    suspend fun getValidAccountNumber(withTime: Boolean = true) =
        // If partner auth is provided, create a new account using the partner API. Otherwise we
        // expect and account with time to be provided.
        if (partnerAuth != null) {
            val accountNumber = partnerClient.createAccount()
            if (withTime) {
                partnerClient.addTime(accountNumber = accountNumber, daysToAdd = 1)
            }
            accountNumber
        } else {
            val validAccountNumber = InstrumentationRegistry.getArguments().getValidAccountNumber()
            mullvadClient.removeAllDevices(validAccountNumber)
            validAccountNumber
        }

    fun getInvalidAccountNumber() = InstrumentationRegistry.getArguments().getInvalidAccountNumber()

    suspend fun tryDeletePartnerAccount(accountNumber: String) =
        if (partnerAuth != null) {
            partnerClient.deleteAccount(accountNumber)
        } else {
            // If we did not create a partner account we should do nothing
        }
}
