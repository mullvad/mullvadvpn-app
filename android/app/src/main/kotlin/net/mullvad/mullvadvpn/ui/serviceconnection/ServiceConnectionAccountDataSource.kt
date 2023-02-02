package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request

class ServiceConnectionAccountDataSource(
    private val connection: Messenger,
    private val dispatcher: EventDispatcher
) {
    val accountCreationResult = callbackFlow {
        val handler: (Event.AccountCreationEvent) -> Unit = { event ->
            trySend(event.result)
        }
        dispatcher.registerHandler(Event.AccountCreationEvent::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    val accountExpiry = callbackFlow {
        val handler: (Event.AccountExpiryEvent) -> Unit = { event ->
            trySend(event.expiry)
        }
        dispatcher.registerHandler(Event.AccountExpiryEvent::class, handler)
        connection.send(Request.FetchAccountExpiry.message)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    val accountHistory = callbackFlow {
        val handler: (Event.AccountHistoryEvent) -> Unit = { event ->
            trySend(event.history)
        }
        dispatcher.registerHandler(Event.AccountHistoryEvent::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    val loginEvents = callbackFlow {
        val handler: (Event.LoginEvent) -> Unit = { event ->
            trySend(event)
        }
        dispatcher.registerHandler(Event.LoginEvent::class, handler)
        awaitClose {
            // The current dispatcher doesn't support unregistration of handlers.
        }
    }

    fun createAccount() = connection.send(Request.CreateAccount.message)
    fun login(accountToken: String) = connection.send(Request.Login(accountToken).message)
    fun logout() = connection.send(Request.Logout.message)
    fun fetchAccountExpiry() = connection.send(Request.FetchAccountExpiry.message)
    fun fetchAccountHistory() = connection.send(Request.FetchAccountHistory.message)
    fun clearAccountHistory() = connection.send(Request.ClearAccountHistory.message)
}
