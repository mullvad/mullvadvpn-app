package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.service.MullvadDaemon

class AuthTokenCache(endpoint: ServiceEndpoint) {
    private var waitingForDaemon = false

    var authToken by observable<String?>(null) { _, _, token ->
        endpoint.sendEvent(Event.AuthToken(token))
    }
        private set

    var daemon by observable<MullvadDaemon?>(null) { _, _, _ ->
        synchronized(this@AuthTokenCache) {
            if (waitingForDaemon) {
                fetchNewToken()
            }
        }
    }

    init {
        endpoint.dispatcher.registerHandler(Request.FetchAuthToken::class) { _ ->
            fetchNewToken()
        }
    }

    fun onDestroy() {
        daemon = null
    }

    private fun fetchNewToken() {
        synchronized(this) {
            val daemon = this.daemon

            if (daemon != null) {
                authToken = daemon.getWwwAuthToken()
                waitingForDaemon = false
            } else {
                waitingForDaemon = true
            }
        }
    }
}
