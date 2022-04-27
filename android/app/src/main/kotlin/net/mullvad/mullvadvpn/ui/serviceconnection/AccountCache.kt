package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime

class AccountCache(private val connection: Messenger, eventDispatcher: EventDispatcher) {
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)
    val onAccountHistoryChange = EventNotifier<String?>(null)
    val onLoginStatusChange = EventNotifier<LoginStatus?>(null)

    private var accountHistory by onAccountHistoryChange.notifiable()
    private var loginStatus by onLoginStatusChange.notifiable()

    private val _accountCreationEvents = MutableSharedFlow<AccountCreationResult>(
        extraBufferCapacity = 1,
        onBufferOverflow = BufferOverflow.DROP_OLDEST
    )
    val accountCreationEvents = _accountCreationEvents.asSharedFlow()

    init {
        eventDispatcher.apply {
            registerHandler(Event.AccountHistory::class) { event ->
                accountHistory = event.history
            }

            registerHandler(Event.LoginStatus::class) { event ->
                loginStatus = event.status
                onAccountExpiryChange.notifyIfChanged(loginStatus?.expiry)
            }

            registerHandler(Event.AccountCreationEvent::class) { event ->
                _accountCreationEvents.tryEmit(event.result)
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

    fun invalidateAccountExpiry(accountExpiryToInvalidate: DateTime) {
        val request = Request.InvalidateAccountExpiry(accountExpiryToInvalidate)

        connection.send(request.message)
    }

    fun clearAccountHistory() {
        connection.send(Request.ClearAccountHistory.message)
    }

    fun onDestroy() {
        onAccountExpiryChange.unsubscribeAll()
        onAccountHistoryChange.unsubscribeAll()
        onLoginStatusChange.unsubscribeAll()
    }
}
