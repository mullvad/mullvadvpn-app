package net.mullvad.mullvadvpn.test.e2e.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.e2e.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.e2e.api.partner.PartnerApi
import net.mullvad.mullvadvpn.test.e2e.constant.getInvalidAccountNumber
import net.mullvad.mullvadvpn.test.e2e.constant.getPartnerAuth
import net.mullvad.mullvadvpn.test.e2e.constant.getValidAccountNumber

interface AccountProvider {
    suspend fun getValidAccountNumber(withTime: Boolean = true): String

    fun getInvalidAccountNumber() = InstrumentationRegistry.getArguments().getInvalidAccountNumber()

    suspend fun cleanup(accountNumber: String)

    companion object {
        fun createAccountProvider(): AccountProvider {
            val partnerAuth: String? = InstrumentationRegistry.getArguments().getPartnerAuth()
            return if (partnerAuth != null) {
                PartnerProvider(partnerAuth)
            } else {
                StaticProvider()
            }
        }
    }
}

data class StaticProvider(private val mullvadApi: MullvadApi = MullvadApi()) : AccountProvider {

    override suspend fun getValidAccountNumber(withTime: Boolean): String =
        InstrumentationRegistry.getArguments().getValidAccountNumber().also {
            mullvadApi.removeAllDevices(it)
        }

    override suspend fun cleanup(accountNumber: String) = Unit // No-op
}

data class PartnerProvider(val partnerApi: PartnerApi) : AccountProvider {
    constructor(partnerAuth: String) : this(partnerApi = PartnerApi(partnerAuth))

    override suspend fun getValidAccountNumber(withTime: Boolean): String =
        partnerApi.createAccount().also {
            if (withTime) {
                partnerApi.addTime(accountNumber = it, daysToAdd = 1)
            }
        }

    override suspend fun cleanup(accountNumber: String) {
        partnerApi.deleteAccount(accountNumber)
    }
}
