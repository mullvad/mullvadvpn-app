package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

class ServiceConnectionAccountDataSource(private val messageHandler: MessageHandler) {
    val accountCreationResult =
        messageHandler.events.filterIsInstance(Event.AccountCreationEvent::class).map { it.result }
    // TODO: We previously also sent a Request here. How should we handle it now?
    val accountExpiry =
        messageHandler.events.filterIsInstance(Event.AccountExpiryEvent::class).map { it.expiry }
    val accountHistory =
        messageHandler.events.filterIsInstance(Event.AccountHistoryEvent::class).map { it.history }
    val loginEvents =
        messageHandler.events.filterIsInstance(Event.LoginEvent::class).map { it.result }

    fun createAccount() = messageHandler.trySendRequest(Request.CreateAccount)

    fun login(accountToken: String) = messageHandler.trySendRequest(Request.Login(accountToken))

    fun logout() = messageHandler.trySendRequest(Request.Logout)

    fun fetchAccountExpiry() = messageHandler.trySendRequest(Request.FetchAccountExpiry)

    fun fetchAccountHistory() = messageHandler.trySendRequest(Request.FetchAccountHistory)

    fun clearAccountHistory() = messageHandler.trySendRequest(Request.ClearAccountHistory)
}
