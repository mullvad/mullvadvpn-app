package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.endpoint.AuthTokenCache.Companion.Command

class AuthTokenCache(endpoint: ServiceEndpoint) : Actor<Command>() {
    companion object {
        enum class Command {
            Fetch
        }
    }

    private val daemon = endpoint.intermittentDaemon

    var authToken by observable<String?>(null) { _, _, token ->
        endpoint.sendEvent(Event.AuthToken(token))
    }
        private set

    init {
        endpoint.dispatcher.registerHandler(Request.FetchAuthToken::class) { _ ->
            sendBlocking(Command.Fetch)
        }
    }

    fun onDestroy() = closeActor()

    override suspend fun onNewCommand(command: Command) = when (command) {
        Command.Fetch -> authToken = daemon.await().getWwwAuthToken()
    }
}
