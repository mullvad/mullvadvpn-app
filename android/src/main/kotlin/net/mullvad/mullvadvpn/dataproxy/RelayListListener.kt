package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.service.MullvadDaemon

class RelayListListener(
    val daemon: Deferred<MullvadDaemon>,
    val settingsListener: SettingsListener
) {
    private val setUpJob = setUp()

    private var relayList: RelayList? = null
    private var relaySettings: RelaySettings? = null

    var selectedRelayItem: RelayItem? = null
        private set

    var onRelayListChange: ((RelayList, RelayItem?) -> Unit)? = null
        set(value) {
            field = value

            synchronized(this) {
                val relayList = this.relayList

                if (relayList != null) {
                    value?.invoke(relayList, selectedRelayItem)
                }
            }
        }

    init {
        settingsListener.onRelaySettingsChange = { newRelaySettings ->
            relaySettingsChanged(newRelaySettings)
        }
    }

    fun onDestroy() {
        setUpJob.cancel()
        settingsListener.onRelaySettingsChange = null

        if (daemon.isActive) {
            daemon.cancel()
        } else {
            daemon.getCompleted().onRelayListChange = null
        }
    }

    private fun setUp() = GlobalScope.launch(Dispatchers.Default) {
        setUpListener()
        fetchInitialRelayList()
    }

    private suspend fun setUpListener() {
        daemon.await().onRelayListChange = { relayLocations ->
            relayListChanged(RelayList(relayLocations))
        }
    }

    private suspend fun fetchInitialRelayList() {
        val relayLocations = daemon.await().getRelayLocations()

        synchronized(this) {
            if (relayList == null) {
                relayListChanged(RelayList(relayLocations))
            }
        }
    }

    private fun relaySettingsChanged(newRelaySettings: RelaySettings?) {
        synchronized(this) {
            val relayList = this.relayList

            relaySettings = newRelaySettings
                ?: RelaySettings.Normal(RelayConstraints(Constraint.Any()))

            if (relayList != null) {
                relayListChanged(relayList)
            }
        }
    }

    private fun relayListChanged(newRelayList: RelayList) {
        synchronized(this) {
            relayList = newRelayList
            selectedRelayItem = findSelectedRelayItem()

            onRelayListChange?.invoke(newRelayList, selectedRelayItem)
        }
    }

    private fun findSelectedRelayItem(): RelayItem? {
        val relaySettings = this.relaySettings

        when (relaySettings) {
            is RelaySettings.CustomTunnelEndpoint -> return null
            is RelaySettings.Normal -> {
                val location = relaySettings.relayConstraints.location

                return relayList?.findItemForLocation(location, true)
            }
        }

        return null
    }
}
