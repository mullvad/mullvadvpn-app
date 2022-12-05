package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request

class AuthTokenCache(endpoint: ServiceEndpoint) {
    companion object {
        private enum class Command {
            Fetch
        }
    }

    private val daemon = endpoint.intermittentDaemon
    private val requestQueue = spawnActor()

    var authToken by observable<String?>(null) { _, _, token ->
        endpoint.sendEvent(Event.AuthToken(token))
    }
        private set

    init {
        endpoint.dispatcher.registerHandler(Request.FetchAuthToken::class) { _ ->
            requestQueue.trySendBlocking(Command.Fetch)
        }
    }

    fun onDestroy() {
        requestQueue.close()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            for (command in channel) {
                when (command) {
                    Command.Fetch -> authToken = daemon.await().getWwwAuthToken()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }
}
