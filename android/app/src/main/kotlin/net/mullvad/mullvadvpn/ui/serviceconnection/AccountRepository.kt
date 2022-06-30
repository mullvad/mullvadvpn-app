package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

class AccountRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val dataSource
        get() = serviceConnectionManager.connectionState.value.readyContainer()?.accountDataSource

    private val _cachedCreatedAccount = MutableStateFlow<String?>(null)
    val cachedCreatedAccount = _cachedCreatedAccount.asStateFlow()

    val accountCreationEvents: SharedFlow<AccountCreationResult> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                state.container.accountDataSource.accountCreationResult
            }
            .onEach {
                _cachedCreatedAccount.value = (it as AccountCreationResult.Success).accountToken
            }
            .shareIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed()
            )

    val accountExpiryState: StateFlow<AccountExpiry> = serviceConnectionManager.connectionState
        .flatMapReadyConnectionOrDefault(flowOf()) { state ->
            state.container.accountDataSource.accountExpiry
        }
        .onStart {
            fetchAccountExpiry()
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            AccountExpiry.Missing
        )

    val accountHistoryEvents: StateFlow<AccountHistory> = serviceConnectionManager.connectionState
        .flatMapReadyConnectionOrDefault(flowOf()) { state ->
            state.container.accountDataSource.accountHistory
        }
        .onStart {
            fetchAccountHistory()
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            AccountHistory.Missing
        )

    val loginEvents: SharedFlow<Event.LoginEvent> = serviceConnectionManager.connectionState
        .flatMapReadyConnectionOrDefault(flowOf()) { state ->
            state.container.accountDataSource.loginEvents
        }
        .shareIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed()
        )

    fun createAccount() {
        dataSource?.createAccount()
    }

    fun login(accountToken: String) {
        dataSource?.login(accountToken)
    }

    fun logout() {
        dataSource?.logout()
    }

    fun fetchAccountExpiry() {
        dataSource?.fetchAccountExpiry()
    }

    fun fetchAccountHistory() {
        dataSource?.fetchAccountHistory()
    }

    fun clearAccountHistory() {
        dataSource?.clearAccountHistory()
    }
}
