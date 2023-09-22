package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.accountDataSource
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

class AccountRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val _cachedCreatedAccount = MutableStateFlow<String?>(null)
    val cachedCreatedAccount = _cachedCreatedAccount.asStateFlow()

    private val accountCreationEvents: SharedFlow<AccountCreationResult> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                state.container.accountDataSource.accountCreationResult
            }
            .onEach {
                _cachedCreatedAccount.value = (it as? AccountCreationResult.Success)?.accountToken
            }
            .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    val accountExpiryState: StateFlow<AccountExpiry> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                state.container.accountDataSource.accountExpiry
            }
            .stateIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed(),
                AccountExpiry.Missing
            )

    val accountHistory: StateFlow<AccountHistory> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                state.container.accountDataSource.accountHistory
            }
            .onStart { fetchAccountHistory() }
            .stateIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed(),
                AccountHistory.Missing
            )

    private val loginEvents: SharedFlow<Event.LoginEvent> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                state.container.accountDataSource.loginEvents
            }
            .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    suspend fun createAccount(): AccountCreationResult =
        withContext(dispatcher) {
            val deferred = async { accountCreationEvents.first() }
            serviceConnectionManager.accountDataSource()?.createAccount()
            deferred.await()
        }

    suspend fun login(accountToken: String): LoginResult =
        withContext(Dispatchers.IO) {
            val deferred = async { loginEvents.first().result }
            serviceConnectionManager.accountDataSource()?.login(accountToken)
            deferred.await()
        }

    fun logout() {
        clearCreatedAccountCache()
        serviceConnectionManager.accountDataSource()?.logout()
    }

    fun fetchAccountExpiry() {
        serviceConnectionManager.accountDataSource()?.fetchAccountExpiry()
    }

    fun fetchAccountHistory() {
        serviceConnectionManager.accountDataSource()?.fetchAccountHistory()
    }

    fun clearAccountHistory() {
        serviceConnectionManager.accountDataSource()?.clearAccountHistory()
    }

    private fun clearCreatedAccountCache() {
        _cachedCreatedAccount.value = null
    }
}
