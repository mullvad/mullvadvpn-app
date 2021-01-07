package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.talpid.util.EventNotifier

class RelayListListener {
    val relayListNotifier = EventNotifier<RelayList?>(null)

    var relayList by relayListNotifier.notifiable()
        private set

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, newDaemon ->
        oldDaemon?.onRelayListChange = null

        if (newDaemon != null) {
            setUpListener(newDaemon)
            fetchInitialRelayList(newDaemon)
        }
    }

    fun onDestroy() {
        relayListNotifier.unsubscribeAll()
        daemon = null
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
