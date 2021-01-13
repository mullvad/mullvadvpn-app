package net.mullvad.mullvadvpn.service

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.util.EventNotifier

class RelayListListener {
    companion object {
        private enum class Command {
            SetRelayLocation,
        }
    }

    private val availableDaemon = Intermittent<MullvadDaemon>()
    private val commandChannel = spawnActor()

    val relayListNotifier = EventNotifier<RelayList?>(null)

    var relayList by relayListNotifier.notifiable()
        private set

    var selectedRelayLocation by observable<LocationConstraint?>(null) { _, _, _ ->
        commandChannel.sendBlocking(Command.SetRelayLocation)
    }

    var daemon by observable<MullvadDaemon?>(null) { _, oldDaemon, newDaemon ->
        oldDaemon?.onRelayListChange = null

        if (newDaemon != null) {
            setUpListener(newDaemon)
            fetchInitialRelayList(newDaemon)
        }

        availableDaemon.spawnUpdate(newDaemon)
    }

    fun onDestroy() {
        relayListNotifier.unsubscribeAll()
        commandChannel.close()
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

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.CONFLATED) {
        try {
            while (true) {
                val command = channel.receive()

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

        availableDaemon.await().updateRelaySettings(update)
    }
}
