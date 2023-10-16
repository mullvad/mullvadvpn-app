package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request

class ServiceConnectionAccountDataSource(private val messageHandler: MessageHandler) {
    val accountCreationResult =
        messageHandler.events<Event.AccountCreationEvent>().map { it.result }
    val accountExpiry = messageHandler.events<Event.AccountExpiryEvent>().map { it.expiry }
    val accountHistory = messageHandler.events<Event.AccountHistoryEvent>().map { it.history }
    val loginEvents = messageHandler.events<Event.LoginEvent>().map { it.result }

    fun createAccount() = messageHandler.trySendRequest(Request.CreateAccount)

    fun login(accountToken: String) = messageHandler.trySendRequest(Request.Login(accountToken))

    fun logout() = messageHandler.trySendRequest(Request.Logout)

    fun fetchAccountExpiry() = messageHandler.trySendRequest(Request.FetchAccountExpiry)

    fun fetchAccountHistory() = messageHandler.trySendRequest(Request.FetchAccountHistory)

    fun clearAccountHistory() = messageHandler.trySendRequest(Request.ClearAccountHistory)
}
