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
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.LoginResult

class AccountRepository(
    private val messageHandler: MessageHandler,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val _cachedCreatedAccount = MutableStateFlow<String?>(null)
    val cachedCreatedAccount = _cachedCreatedAccount.asStateFlow()

    private val accountCreationEvents: SharedFlow<AccountCreationResult> =
        messageHandler
            .events<Event.AccountCreationEvent>()
            .map { it.result }
            .onEach {
                _cachedCreatedAccount.value = (it as? AccountCreationResult.Success)?.accountToken
            }
            .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    val accountExpiryState: StateFlow<AccountExpiry> =
        messageHandler
            .events<Event.AccountExpiryEvent>()
            .map { it.expiry }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, AccountExpiry.Missing)

    val accountHistory: StateFlow<AccountHistory> =
        messageHandler
            .events<Event.AccountHistoryEvent>()
            .map { it.history }
            .onStart { fetchAccountHistory() }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Lazily, AccountHistory.Missing)

    private val loginEvents: SharedFlow<LoginResult> =
        messageHandler
            .events<Event.LoginEvent>()
            .map { it.result }
            .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    suspend fun createAccount(): AccountCreationResult =
        withContext(dispatcher) {
            val deferred = async { accountCreationEvents.first() }
            messageHandler.trySendRequest(Request.CreateAccount)
            deferred.await().also { fetchAccountHistory() }
        }

    suspend fun login(accountToken: String): LoginResult =
        withContext(Dispatchers.IO) {
            val deferred = async { loginEvents.first() }
            messageHandler.trySendRequest(Request.Login(accountToken))
            deferred.await().also { fetchAccountHistory() }
        }

    fun logout() {
        clearCreatedAccountCache()
        messageHandler.trySendRequest(Request.Logout)
    }

    fun fetchAccountExpiry() {
        messageHandler.trySendRequest(Request.FetchAccountExpiry)
    }

    fun fetchAccountHistory() {
        messageHandler.trySendRequest(Request.FetchAccountHistory)
    }

    fun clearAccountHistory() {
        messageHandler.trySendRequest(Request.ClearAccountHistory)
    }

    fun clearCreatedAccountCache() {
        _cachedCreatedAccount.value = null
    }
}
