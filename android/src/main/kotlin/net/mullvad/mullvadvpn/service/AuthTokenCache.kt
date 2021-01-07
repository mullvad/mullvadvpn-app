package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import net.mullvad.talpid.util.EventNotifier

class AuthTokenCache {
    private var waitingForDaemon = false

    val authTokenNotifier = EventNotifier<String?>(null)

    var authToken by authTokenNotifier.notifiable()
        private set

    var daemon by observable<MullvadDaemon?>(null) { _, _, _ ->
        synchronized(this@AuthTokenCache) {
            if (waitingForDaemon) {
                fetchNewToken()
            }
        }
    }

    fun fetchNewToken() {
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

    fun onDestroy() {
        daemon = null
        authTokenNotifier.unsubscribeAll()
    }
}
