package net.mullvad.mullvadvpn.test.api.misc

import androidx.test.platform.app.InstrumentationRegistry
import net.mullvad.mullvadvpn.test.api.constant.getInvalidAccountNumber
import net.mullvad.mullvadvpn.test.api.constant.getPartnerAuth
import net.mullvad.mullvadvpn.test.api.constant.getValidAccountNumber
import net.mullvad.mullvadvpn.test.api.mullvad.MullvadApi
import net.mullvad.mullvadvpn.test.api.mullvad.removeAllDevices
import net.mullvad.mullvadvpn.test.api.partner.PartnerApi

interface AccountProvider {
    suspend fun getValidAccountNumber(withTime: Boolean = true): String

    fun getInvalidAccountNumber(infrastructure: String) =
        InstrumentationRegistry.getArguments().getInvalidAccountNumber(infrastructure)

    suspend fun cleanup(accountNumber: String)

    companion object {
        fun createAccountProvider(infrastructure: String, baseDomain: String): AccountProvider {
            val partnerAuth: String? =
                InstrumentationRegistry.getArguments().getPartnerAuth(infrastructure)
            return if (partnerAuth != null) {
                PartnerProvider(partnerAuth = partnerAuth, baseDomain = baseDomain)
            } else {
                StaticProvider(baseDomain, infrastructure)
            }
        }
    }
}

data class StaticProvider(
    private val baseDomain: String,
    private val infrastructure: String,
    private val mullvadApi: MullvadApi = MullvadApi(baseDomain),
) : AccountProvider {

    override suspend fun getValidAccountNumber(withTime: Boolean): String =
        InstrumentationRegistry.getArguments().getValidAccountNumber(infrastructure).also {
            mullvadApi.removeAllDevices(it)
        }

    override suspend fun cleanup(accountNumber: String) = Unit // No-op
}

data class PartnerProvider(val partnerApi: PartnerApi) : AccountProvider {
    constructor(
        partnerAuth: String,
        baseDomain: String,
    ) : this(partnerApi = PartnerApi(partnerAuth, baseDomain))

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
