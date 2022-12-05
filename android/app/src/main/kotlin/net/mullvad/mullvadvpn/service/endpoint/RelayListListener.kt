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
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.service.MullvadDaemon

class RelayListListener(endpoint: ServiceEndpoint) {
    companion object {
        private enum class Command {
            SetRelayLocation,
        }
    }

    private val commandChannel = spawnActor()
    private val daemon = endpoint.intermittentDaemon

    private var selectedRelayLocation by observable<LocationConstraint?>(null) { _, _, _ ->
        commandChannel.trySendBlocking(Command.SetRelayLocation)
    }

    var relayList by observable<RelayList?>(null) { _, _, relays ->
        endpoint.sendEvent(Event.NewRelayList(relays))
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
        commandChannel.close()
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

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.CONFLATED) {
        try {
            for (command in channel) {
                when (command) {
                    Command.SetRelayLocation -> updateRelayConstraints()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }

    private suspend fun updateRelayConstraints() {
        val constraint: Constraint<LocationConstraint> = selectedRelayLocation?.let { location ->
            Constraint.Only(location)
        } ?: Constraint.Any()

        val update = RelaySettingsUpdate.Normal(RelayConstraintsUpdate(constraint))

        daemon.await().updateRelaySettings(update)
    }
}
