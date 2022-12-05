package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList

class RelayListListener(
    private val connection: Messenger,
    eventDispatcher: EventDispatcher,
    private val settingsListener: SettingsListener
) {
    private var relayList: RelayList? = null
    private var relaySettings: RelaySettings? = null

    var selectedRelayItem: RelayItem? = null
        private set

    var selectedRelayLocation: LocationConstraint?
        get() {
            val settings = relaySettings as? RelaySettings.Normal
            val location = settings?.relayConstraints?.location as? Constraint.Only

            return location?.value
        }
        set(value) {
            connection.send(Request.SetRelayLocation(value).message)
        }

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
        eventDispatcher.registerHandler(Event.NewRelayList::class) { event ->
            event.relayList?.let { relayLocations ->
                relayListChanged(RelayList(relayLocations))
            }
        }

        settingsListener.relaySettingsNotifier.subscribe(this) { newRelaySettings ->
            relaySettingsChanged(newRelaySettings)
        }
    }

    fun onDestroy() {
        settingsListener.relaySettingsNotifier.unsubscribe(this)
        onRelayListChange = null
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
            else -> { /* NOOP */ }
        }

        return null
    }
}
