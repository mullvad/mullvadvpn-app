package net.mullvad.mullvadvpn.lib.shared

import arrow.core.Either
import arrow.core.raise.nullable
import java.time.ZonedDateTime
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
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

    private val _mutableAccountDataCache: MutableSharedFlow<AccountData> = MutableSharedFlow()

    private val _isNewAccount: MutableStateFlow<Boolean> = MutableStateFlow(false)

    private val _mutableAccountHistory: MutableStateFlow<AccountNumber?> = MutableStateFlow(null)

    val isNewAccount: StateFlow<Boolean> = _isNewAccount

    val accountHistory: StateFlow<AccountNumber?> = _mutableAccountHistory

    val accountData: StateFlow<AccountData?> =
        merge(
                managementService.deviceState.filterNotNull().map { deviceState ->
                    when (deviceState) {
                        is DeviceState.LoggedIn -> {
                            managementService.getAccountData(deviceState.accountNumber).getOrNull()
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

    suspend fun getAccountData(): AccountData? = nullable {
        val deviceState = ensureNotNull(deviceRepository.deviceState.value as? DeviceState.LoggedIn)

        val accountData =
            managementService.getAccountData(deviceState.accountNumber).getOrNull().bind()

        // Update stateflow cache
        _mutableAccountDataCache.emit(accountData)
        accountData
    }

    suspend fun getWebsiteAuthToken(): WebsiteAuthToken? =
        managementService.getWebsiteAuthToken().getOrNull()

    internal suspend fun onVoucherRedeemed(newExpiry: ZonedDateTime) {
        accountData.value?.copy(expiryDate = newExpiry)?.let { _mutableAccountDataCache.emit(it) }
    }
}
