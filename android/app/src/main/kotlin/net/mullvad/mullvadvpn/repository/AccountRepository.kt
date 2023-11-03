package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.AccountState
import net.mullvad.mullvadvpn.model.LoginResult

class AccountRepository(
    private val managementService: ManagementService,
    val scope: CoroutineScope
) {
    private val _cachedCreatedAccount = MutableStateFlow<String?>(null)
    val cachedCreatedAccount = _cachedCreatedAccount.asStateFlow()

    val accountState =
        managementService.deviceState.stateIn(
            scope = scope,
            SharingStarted.Eagerly,
            AccountState.Unrecognized
        )

    private val _mutableAccountHistory: MutableStateFlow<AccountHistory> =
        MutableStateFlow(AccountHistory.Missing)
    val accountHistory: StateFlow<AccountHistory> = _mutableAccountHistory

    private val _mutableAccountExpiry: MutableStateFlow<AccountExpiry> =
        MutableStateFlow(AccountExpiry.Missing)
    val accountExpiry: StateFlow<AccountExpiry> = _mutableAccountExpiry

    suspend fun createAccount(): AccountCreationResult = managementService.createAccount()

    suspend fun login(accountToken: String): LoginResult =
        managementService.loginAccount(accountToken)

    suspend fun logout() = managementService.logoutAccount()

    suspend fun fetchAccountHistory() {
        _mutableAccountHistory.update { managementService.getAccountHistory() }
    }

    suspend fun clearAccountHistory() {
        managementService.clearAccountHistory()
        fetchAccountHistory()
    }

    suspend fun getAccountExpiry(): AccountExpiry {
        val accountExpiry =
            managementService.getAccountExpiry(
                (accountState.value as AccountState.LoggedIn).accountToken
            )
        _mutableAccountExpiry.update { accountExpiry }
        return accountExpiry
    }
}
