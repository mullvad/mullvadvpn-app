package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime

class AccountCache(private val connection: Messenger, eventDispatcher: EventDispatcher) {
    val onAccountNumberChange = EventNotifier<String?>(null)
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)
    val onAccountHistoryChange = EventNotifier<String?>(null)
    val onLoginStatusChange = EventNotifier<LoginStatus?>(null)

    private var accountHistory by onAccountHistoryChange.notifiable()
    private var loginStatus by onLoginStatusChange.notifiable()

    init {
        eventDispatcher.apply {
            registerHandler(Event.AccountHistory::class) { event ->
                accountHistory = event.history
            }

            registerHandler(Event.LoginStatus::class) { event ->
                loginStatus = event.status

                onAccountNumberChange.notifyIfChanged(loginStatus?.account)
                onAccountExpiryChange.notifyIfChanged(loginStatus?.expiry)
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
        onAccountNumberChange.unsubscribeAll()
        onAccountExpiryChange.unsubscribeAll()
        onAccountHistoryChange.unsubscribeAll()
        onLoginStatusChange.unsubscribeAll()
    }
}
