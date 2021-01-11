package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import java.util.LinkedList
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.service.Event
import net.mullvad.mullvadvpn.service.Request
import net.mullvad.mullvadvpn.util.DispatchingHandler

class AuthTokenCache(val connection: Messenger, eventDispatcher: DispatchingHandler<Event>) {
    private val fetchQueue = LinkedList<CompletableDeferred<String>>()

    init {
        eventDispatcher.registerHandler(Event.AuthToken::class) { event ->
            synchronized(this@AuthTokenCache) {
                if (!fetchQueue.isEmpty()) {
                    fetchQueue.removeFirst()?.complete(event.token ?: "")
                }
            }
        }
    }

    suspend fun fetchAuthToken(): String {
        val authToken = CompletableDeferred<String>()

        synchronized(this) {
            fetchQueue.addLast(authToken)
        }

        connection.send(Request.FetchAuthToken().message)

        return authToken.await()
    }

    fun onDestroy() {
        for (pendingFetch in fetchQueue) {
            pendingFetch.cancel()
        }

        fetchQueue.clear()
    }
}
