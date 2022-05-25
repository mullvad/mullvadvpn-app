package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.AccountHistory

class AccountCache(private val connection: Messenger, eventDispatcher: EventDispatcher) {

    private val _accountCreationEvents = MutableSharedFlow<AccountCreationResult>(
        extraBufferCapacity = 1,
        onBufferOverflow = BufferOverflow.DROP_OLDEST
    )
    val accountCreationEvents = _accountCreationEvents.asSharedFlow()

    private val _accountExpiryState = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    val accountExpiryState = _accountExpiryState.asStateFlow()

    private val _accountHistoryEvents = MutableSharedFlow<AccountHistory>(
        extraBufferCapacity = 1,
        onBufferOverflow = BufferOverflow.DROP_OLDEST
    )
    val accountHistoryEvents = _accountHistoryEvents.asSharedFlow()

    private val _loginEvents = MutableSharedFlow<Event.LoginEvent>(
        extraBufferCapacity = 1,
        onBufferOverflow = BufferOverflow.DROP_OLDEST
    )
    val loginEvents = _loginEvents.asSharedFlow()

    init {
        eventDispatcher.apply {
            registerHandler(Event.AccountCreationEvent::class) { event ->
                _accountCreationEvents.tryEmit(event.result)
            }

            registerHandler(Event.AccountExpiryEvent::class) { event ->
                _accountExpiryState.tryEmit(event.expiry)
            }

            registerHandler(Event.AccountHistoryEvent::class) { event ->
                _accountHistoryEvents.tryEmit(event.history)
            }

            registerHandler(Event.LoginEvent::class) { event ->
                _loginEvents.tryEmit(event)
            }
        }
    }

    fun createNewAccount() {
        connection.send(Request.CreateAccount.message)
    }

    fun login(account: String) {
        connection.send(Request.Login(account).message)
    }

    fun logout() {
        connection.send(Request.Logout.message)
    }

    fun fetchAccountExpiry() {
        connection.send(Request.FetchAccountExpiry.message)
    }

    fun fetchAccountHistory() {
        connection.send(Request.FetchAccountHistory.message)
    }

    fun clearAccountHistory() {
        connection.send(Request.ClearAccountHistory.message)
    }
}
