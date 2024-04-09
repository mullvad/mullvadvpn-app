package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.LoginResult

class AccountRepository(
    private val managementService: ManagementService,
    val scope: CoroutineScope
) {
    val accountState =
        managementService.deviceState.stateIn(
            scope = scope,
            SharingStarted.Eagerly,
            DeviceState.Unknown
        )

    private val _mutableAccountData: MutableStateFlow<AccountData?> = MutableStateFlow(null)
    val accountData: StateFlow<AccountData?> = _mutableAccountData

    suspend fun createAccount(): AccountCreationResult = managementService.createAccount()

    suspend fun login(accountToken: String): LoginResult =
        managementService.loginAccount(accountToken)

    suspend fun logout() {
        managementService.logoutAccount()
        getAccountAccountData()
    }

    suspend fun fetchAccountHistory(): AccountToken? = managementService.getAccountHistory()

    suspend fun clearAccountHistory() = managementService.clearAccountHistory()

    // TODO improve this to account for different logged in state properly (E.g test what
    // AccountData will reply with)
    suspend fun getAccountAccountData(): AccountData? {
        val accountData =
            if (accountState.value !is DeviceState.LoggedIn) null
            else {
                managementService.getAccountData(
                    (accountState.value as DeviceState.LoggedIn).accountToken
                )
            }
        _mutableAccountData.update { accountData }
        return accountData
    }
}
