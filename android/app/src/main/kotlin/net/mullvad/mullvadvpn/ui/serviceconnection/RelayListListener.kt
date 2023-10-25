package net.mullvad.mullvadvpn.ui.serviceconnection

import android.os.Messenger
import net.mullvad.mullvadvpn.lib.common.util.toGeographicLocationConstraint
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.findItemForLocation
import net.mullvad.mullvadvpn.relaylist.toRelayCountries

class RelayListListener(
    private val connection: Messenger,
    eventDispatcher: EventDispatcher,
    private val settingsListener: SettingsListener
) {
    private var relayCountries: List<RelayCountry>? = null
    private var relaySettings: RelaySettings? = null
    private var portRanges: List<PortRange> = emptyList()

    var selectedRelayItem: RelayItem? = null
        private set

    var selectedRelayLocation: GeographicLocationConstraint?
        get() {
            val settings = relaySettings as? RelaySettings.Normal
            val location = settings?.relayConstraints?.location as? Constraint.Only

            return location?.value?.toGeographicLocationConstraint()
        }
        set(value) {
            connection.send(Request.SetRelayLocation(value).message)
        }

    var selectedWireguardConstraints: WireguardConstraints?
        get() {
            val settings = relaySettings as? RelaySettings.Normal

            return settings?.relayConstraints?.wireguardConstraints?.port?.let { port ->
                WireguardConstraints(port)
            }
        }
        set(value) {
            connection.send(Request.SetWireguardConstraints(value).message)
        }

    var onRelayCountriesChange: ((List<RelayCountry>, RelayItem?) -> Unit)? = null
        set(value) {
            field = value

            synchronized(this) {
                val relayCountries = this.relayCountries

                if (relayCountries != null) {
                    value?.invoke(relayCountries, selectedRelayItem)
                }
            }
        }

    var onPortRangesChange: ((List<PortRange>) -> Unit)? = null
        set(value) {
            field = value

            synchronized(this) { value?.invoke(portRanges) }
        }

    init {
        eventDispatcher.registerHandler(Event.NewRelayList::class) { event ->
            event.relayList?.let { relayLocations ->
                relayListChanged(relayLocations.toRelayCountries())
                portRangesChanged(relayLocations.wireguardEndpointData.portRanges)
            }
        }

        settingsListener.relaySettingsNotifier.subscribe(this) { newRelaySettings ->
            relaySettingsChanged(newRelaySettings)
        }
    }

    fun onDestroy() {
        settingsListener.relaySettingsNotifier.unsubscribe(this)
        onRelayCountriesChange = null
    }

    private fun relaySettingsChanged(newRelaySettings: RelaySettings?) {
        synchronized(this) {
            val relayCountries = this.relayCountries
            val portRanges = this.portRanges

            relaySettings =
                newRelaySettings
                    ?: RelaySettings.Normal(
                        RelayConstraints(
                            location = Constraint.Any(),
                            ownership = Constraint.Any(),
                            wireguardConstraints = WireguardConstraints(Constraint.Any()),
                            providers = Constraint.Any()
                        )
                    )

            if (relayCountries != null) {
                relayListChanged(relayCountries)
            }
            portRangesChanged(portRanges)
        }
    }

    private fun relayListChanged(newRelayCountries: List<RelayCountry>) {
        synchronized(this) {
            relayCountries = newRelayCountries
            selectedRelayItem = findSelectedRelayItem()

            onRelayCountriesChange?.invoke(newRelayCountries, selectedRelayItem)
        }
    }

    private fun portRangesChanged(newPortRanges: List<PortRange>) {
        synchronized(this) {
            portRanges = newPortRanges

            onPortRangesChange?.invoke(portRanges)
        }
    }

    private fun findSelectedRelayItem(): RelayItem? {
        val relaySettings = this.relaySettings

        when (relaySettings) {
            is RelaySettings.CustomTunnelEndpoint -> return null
            is RelaySettings.Normal -> {
                val location = relaySettings.relayConstraints.location
                return relayCountries?.findItemForLocation(
                    location.toGeographicLocationConstraint()
                )
            }
            else -> {
                /* NOOP */
            }
        }

        return null
    }
}
