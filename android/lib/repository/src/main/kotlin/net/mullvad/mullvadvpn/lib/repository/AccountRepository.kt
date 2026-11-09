package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import java.time.ZonedDateTime
import kotlin.time.Duration
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.ClearAccountHistoryError
import net.mullvad.mullvadvpn.lib.model.CreateAccountError
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

class AccountRepository(
    private val managementService: ManagementService,
    private val deviceRepository: DeviceRepository,
) {

    private val _isNewAccount: MutableStateFlow<Boolean> = MutableStateFlow(false)
    val isNewAccount: StateFlow<Boolean> = _isNewAccount

    private val _mutableAccountHistory: MutableStateFlow<AccountNumber?> = MutableStateFlow(null)
    val accountHistory: StateFlow<AccountNumber?> = _mutableAccountHistory

    private val _mutableAccountData: MutableStateFlow<AccountData?> = MutableStateFlow(null)
    val accountData: StateFlow<AccountData?> = _mutableAccountData

    private var lastSuccessfulAccountDataFetch: ZonedDateTime? = null

    suspend fun createAccount(): Either<CreateAccountError, AccountNumber> =
        managementService.createAccount().onRight { _isNewAccount.update { true } }

    suspend fun login(accountNumber: AccountNumber): Either<LoginAccountError, Unit> =
        managementService.loginAccount(accountNumber)

    suspend fun logout() =
        managementService.logoutAccount().onRight {
            _mutableAccountData.update { null }
            _isNewAccount.update { false }
        }

    suspend fun fetchAccountHistory(): AccountNumber? =
        managementService
            .getAccountHistory()
            .onRight { _mutableAccountHistory.value = it }
            .getOrNull()

    suspend fun clearAccountHistory(): Either<ClearAccountHistoryError, Unit> =
        managementService.clearAccountHistory().onRight { _mutableAccountHistory.value = null }

    /*
     * Fetches the account data from the server, and updates the cache.
     * Unless ignoreLastSuccessful is true, it will only fetch if no fetch was made in the last minute.
     */
    suspend fun fetchAccountData(
        ignoreLastSuccessful: Boolean = true,
        timeout: Duration = Duration.INFINITE,
    ): AccountData? =
        withTimeoutOrNull(timeout) {
            // Only refresh if logged in
            val deviceState =
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>().first()

            if (ignoreLastSuccessful || lastSuccessfulAccountDataFetch.canFetchAccountData()) {
                val accountData =
                    managementService
                        .getAccountData(deviceState.accountNumber)
                        .onRight { lastSuccessfulAccountDataFetch = ZonedDateTime.now() }
                        .getOrNull()

                // Update stateflow cache and return value, only update and return if device state
                // is still logged in and using the same account number
                deviceRepository.deviceState.value?.let {
                    if (
                        it is DeviceState.LoggedIn && it.accountNumber == accountData?.accountNumber
                    ) {
                        _mutableAccountData.emit(accountData)
                        return@withTimeoutOrNull accountData
                    }
                }
            }
            null
        }

    suspend fun getWebsiteAuthToken(): WebsiteAuthToken? =
        managementService.getWebsiteAuthToken().getOrNull()

    internal suspend fun onVoucherRedeemed(newExpiry: ZonedDateTime) {
        accountData.value?.copy(expiryDate = newExpiry)?.let { _mutableAccountData.emit(it) }
    }

    fun resetIsNewAccount() {
        _isNewAccount.value = false
    }

    suspend fun deleteAccount(): Either<DeleteAccountError, Unit> =
        managementService.deleteAccount().onRight { _mutableAccountData.value = null }

    private fun ZonedDateTime?.canFetchAccountData(): Boolean =
        this == null || this.isBefore(ZonedDateTime.now().minusMinutes(1))
}
