package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.ipc.Event.NewRelayList
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.endpoint.RelayListListener.Companion.Command

class RelayListListener(endpoint: ServiceEndpoint) : Actor<Command>() {
    companion object {
        enum class Command {
            SetRelayLocation,
        }
    }

    private val daemon = endpoint.intermittentDaemon

    private var selectedRelayLocation by observable<LocationConstraint?>(null) { _, _, _ ->
        sendBlocking(Command.SetRelayLocation)
    }

    var relayList by observable<RelayList?>(null) { _, _, relays ->
        endpoint.sendEvent(NewRelayList(relays))
    }
        private set

    init {
        daemon.registerListener(this) { newDaemon ->
            newDaemon?.let { daemon ->
                setUpListener(daemon)
                fetchInitialRelayList(daemon)
            }
        }

        endpoint.dispatcher.registerHandler(Request.SetRelayLocation::class) { request ->
            selectedRelayLocation = request.relayLocation
        }
    }

    fun onDestroy() {
        closeActor()
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

    override suspend fun onNewCommand(command: Command) = when (command) {
        Command.SetRelayLocation -> updateRelayConstraints()
    }

    private suspend fun updateRelayConstraints() {
        val constraint: Constraint<LocationConstraint> = selectedRelayLocation?.let { location ->
            Constraint.Only(location)
        } ?: Constraint.Any()

        val update = RelaySettingsUpdate.Normal(RelayConstraintsUpdate(constraint))

        daemon.await().updateRelaySettings(update)
    }
}
