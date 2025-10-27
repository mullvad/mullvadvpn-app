package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import java.time.ZonedDateTime
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.ClearAccountHistoryError
import net.mullvad.mullvadvpn.lib.model.CreateAccountError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken

class AccountRepository(
    private val managementService: ManagementService,
    private val deviceRepository: DeviceRepository,
    val scope: CoroutineScope,
) {
    private var lastSuccessfulAccountDataFetch: ZonedDateTime? = null

    private val _mutableAccountDataCache: MutableSharedFlow<AccountData> = MutableSharedFlow()

    private val _isNewAccount: MutableStateFlow<Boolean> = MutableStateFlow(false)

    private val _mutableAccountHistory: MutableStateFlow<AccountNumber?> = MutableStateFlow(null)

    val isNewAccount: StateFlow<Boolean> = _isNewAccount

    val accountHistory: StateFlow<AccountNumber?> = _mutableAccountHistory

    val accountData: StateFlow<AccountData?> =
        merge(
                managementService.deviceState.map { deviceState ->
                    when (deviceState) {
                        is DeviceState.LoggedIn -> {
                            managementService
                                .getAccountData(deviceState.accountNumber)
                                .getOrNull()
                                ?.also { lastSuccessfulAccountDataFetch = ZonedDateTime.now() }
                        }
                        DeviceState.LoggedOut,
                        DeviceState.Revoked -> null
                    }
                },
                _mutableAccountDataCache,
            )
            .distinctUntilChanged()
            .stateIn(scope = scope, SharingStarted.Eagerly, null)

    suspend fun createAccount(): Either<CreateAccountError, AccountNumber> =
        managementService.createAccount().onRight { _isNewAccount.update { true } }

    suspend fun login(accountNumber: AccountNumber): Either<LoginAccountError, Unit> =
        managementService.loginAccount(accountNumber)

    suspend fun logout() =
        managementService.logoutAccount().onRight { _isNewAccount.update { false } }

    suspend fun fetchAccountHistory(): AccountNumber? =
        managementService
            .getAccountHistory()
            .onRight { _mutableAccountHistory.value = it }
            .getOrNull()

    suspend fun clearAccountHistory(): Either<ClearAccountHistoryError, Unit> =
        managementService.clearAccountHistory().onRight { _mutableAccountHistory.value = null }

    /*
     * Fetches the account data from the server, and updates the cache.
     * Unless force is true, it will only fetch if no fetch was made in the last minute.
     */
    suspend fun refreshAccountData(ignoreTimeout: Boolean = true) {
        // Only refresh if logged in
        val deviceState = deviceRepository.deviceState.value as? DeviceState.LoggedIn ?: return

        if (ignoreTimeout || lastSuccessfulAccountDataFetch.canFetchAccountData()) {
            val accountData =
                managementService.getAccountData(deviceState.accountNumber).getOrNull()
            lastSuccessfulAccountDataFetch = ZonedDateTime.now()

            // Update stateflow cache, only update if device state is still logged in and using the
            // same account number
            deviceRepository.deviceState.value?.let {
                if (it is DeviceState.LoggedIn && it.accountNumber == accountData?.accountNumber) {
                    _mutableAccountDataCache.emit(accountData)
                }
            }
        }
    }

    suspend fun getWebsiteAuthToken(): WebsiteAuthToken? =
        managementService.getWebsiteAuthToken().getOrNull()

    internal suspend fun onVoucherRedeemed(newExpiry: ZonedDateTime) {
        accountData.value?.copy(expiryDate = newExpiry)?.let { _mutableAccountDataCache.emit(it) }
    }

    fun resetIsNewAccount() {
        _isNewAccount.value = false
    }

    private fun ZonedDateTime?.canFetchAccountData(): Boolean =
        this == null || this.isBefore(ZonedDateTime.now().minusMinutes(1))
}
