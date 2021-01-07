package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.model.RelayList

class RelayListListener(endpoint: ServiceEndpoint) {
    val daemon = endpoint.intermittentDaemon

    var relayList: RelayList? = null
        private set

    init {
        daemon.registerListener(this) { newDaemon ->
            newDaemon?.let { daemon ->
                setUpListener(daemon)
                fetchInitialRelayList(daemon)
            }
        }
    }

    fun onDestroy() {
        daemon.unregisterListener(this)
    }

    private fun setUpListener(daemon: MullvadDaemon) {
        daemon.onRelayListChange = { relayLocations ->
            relayList = relayLocations
        }
    }

    private fun fetchInitialRelayList(daemon: MullvadDaemon) {
        synchronized(this) {
            if (relayList == null) {
                relayList = daemon.getRelayLocations()
            }
        }
    }
}
