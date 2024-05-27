package net.mullvad.mullvadvpn.lib.shared

import arrow.core.Either
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountToken
import net.mullvad.mullvadvpn.lib.model.CreateAccountError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import org.joda.time.DateTime

class AccountRepository(
    private val managementService: ManagementService,
    val scope: CoroutineScope
) {
    val accountState =
        managementService.deviceState.stateIn(scope = scope, SharingStarted.Eagerly, null)

    private val _mutableAccountData: MutableSharedFlow<AccountData> = MutableSharedFlow()

    private val _isNewAccount: MutableStateFlow<Boolean> = MutableStateFlow(false)
    val isNewAccount: StateFlow<Boolean> = _isNewAccount
    val accountData: StateFlow<AccountData?> =
        merge(
                accountState.filterNotNull().map { deviceState ->
                    when (deviceState) {
                        is DeviceState.LoggedIn -> {
                            managementService.getAccountData(deviceState.accountToken).getOrNull()
                        }
                        DeviceState.LoggedOut,
                        DeviceState.Revoked -> null
                    }
                },
                _mutableAccountData
            )
            .stateIn(scope = scope, SharingStarted.Eagerly, null)

    suspend fun createAccount(): Either<CreateAccountError, AccountToken> =
        managementService.createAccount().onRight { _isNewAccount.update { true } }

    suspend fun login(accountToken: AccountToken): Either<LoginAccountError, Unit> =
        managementService.loginAccount(accountToken)

    suspend fun logout() {
        managementService.logoutAccount()
        getAccountData()
        _isNewAccount.update { false }
    }

    suspend fun fetchAccountHistory(): AccountToken? =
        managementService.getAccountHistory().getOrNull()

    suspend fun clearAccountHistory() = managementService.clearAccountHistory()

    suspend fun getAccountData(): AccountData? {
        val accountData =
            if (accountState.value !is DeviceState.LoggedIn) null
            else {
                managementService
                    .getAccountData((accountState.value as DeviceState.LoggedIn).accountToken)
                    .getOrNull()
            }
        if (accountData != null) {
            _mutableAccountData.emit(accountData)
        }
        return accountData
    }

    fun getAccountToken(): AccountToken? {
        return when (val deviceState = accountState.value) {
            is DeviceState.LoggedIn -> deviceState.accountToken
            else -> null
        }
    }

    suspend fun getWwwAuthToken(): WebsiteAuthToken? = managementService.getWwwAuthToken().getOrNull()

    internal suspend fun onVoucherRedeemed(newExpiry: DateTime) {
        accountData.value?.copy(expiryDate = newExpiry)?.let { _mutableAccountData.emit(it) }
    }
}
