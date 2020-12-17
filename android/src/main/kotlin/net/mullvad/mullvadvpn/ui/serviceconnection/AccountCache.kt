package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.model.LoginStatus
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.talpid.util.EventNotifier
import org.joda.time.DateTime

class AccountCache(val connection: Messenger, eventDispatcher: EventDispatcher) {
    val onAccountNumberChange = EventNotifier<String?>(null)
    val onAccountExpiryChange = EventNotifier<DateTime?>(null)
    val onAccountHistoryChange = EventNotifier<ArrayList<String>>(ArrayList())
    val onLoginStatusChange = EventNotifier<LoginStatus?>(null)

    private var accountHistory by onAccountHistoryChange.notifiable()
    private var loginStatus by onLoginStatusChange.notifiable()

    init {
        eventDispatcher.apply {
            registerHandler(Event.Type.AccountHistory) { event: Event.AccountHistory ->
                accountHistory = event.history ?: ArrayList()
            }

            registerHandler(Event.Type.LoginStatus) { event: Event.LoginStatus ->
                loginStatus = event.status

                onAccountNumberChange.notifyIfChanged(loginStatus?.account)
                onAccountExpiryChange.notifyIfChanged(loginStatus?.expiry)
            }
        }
    }

    fun createNewAccount() {
        connection.send(Request.CreateAccount().message)
    }

    fun login(account: String) {
        connection.send(Request.Login(account).message)
    }

    fun logout() {
        connection.send(Request.Logout().message)
    }

    fun fetchAccountExpiry() {
        connection.send(Request.FetchAccountExpiry().message)
    }

    fun invalidateAccountExpiry(accountExpiryToInvalidate: DateTime) {
        val request = Request.InvalidateAccountExpiry(accountExpiryToInvalidate)

        connection.send(request.message)
    }

    fun removeAccountFromHistory(account: String) {
        connection.send(Request.RemoveAccountFromHistory(account).message)
    }
}
