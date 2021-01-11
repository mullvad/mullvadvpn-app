package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import java.util.LinkedList
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.ipc.DispatchingHandler
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request

class AuthTokenCache(val connection: Messenger, eventDispatcher: DispatchingHandler<Event>) {
    private val fetchQueue = LinkedList<CompletableDeferred<String>>()

    init {
        eventDispatcher.registerHandler(Event.AuthToken::class) { event ->
            synchronized(this@AuthTokenCache) {
                fetchQueue.poll()?.complete(event.token ?: "")
            }
        }
    }

    suspend fun fetchAuthToken(): String {
        val authToken = CompletableDeferred<String>()

        synchronized(this) {
            fetchQueue.offer(authToken)
        }

        connection.send(Request.FetchAuthToken.message)

        return authToken.await()
    }

    fun onDestroy() {
        synchronized(this) {
            for (pendingFetch in fetchQueue) {
                pendingFetch.cancel()
            }

            fetchQueue.clear()
        }
    }
}
