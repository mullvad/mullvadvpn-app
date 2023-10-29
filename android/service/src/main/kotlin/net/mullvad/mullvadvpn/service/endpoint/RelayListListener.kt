package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraintsUpdate
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.service.MullvadDaemon

class RelayListListener(
    endpoint: ServiceEndpoint,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + dispatcher)
    private val daemon = endpoint.intermittentDaemon

    var relayList by
        observable<RelayList?>(null) { _, _, relays ->
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

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.SetRelayLocation>()
                .collect { request ->
                    val update =
                        RelayConstraintsUpdate(
                            location =
                                Constraint.Only(LocationConstraint.Location(request.relayLocation)),
                            providers = null,
                            ownership = null,
                            wireguardConstraints = null
                        )
                    daemon.await().updateRelaySettings(RelaySettingsUpdate.Normal(update))
                }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.SetWireguardConstraints>()
                .collect { request ->
                    val update =
                        RelayConstraintsUpdate(
                            location = null,
                            providers = null,
                            ownership = null,
                            wireguardConstraints = request.wireguardConstraints
                        )
                    daemon.await().updateRelaySettings(RelaySettingsUpdate.Normal(update))
                }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages.filterIsInstance<Request.SetOwnership>().collect {
                request ->
                val update =
                    RelayConstraintsUpdate(
                        location = null,
                        providers = null,
                        ownership = request.ownership,
                        wireguardConstraints = null
                    )
                daemon.await().updateRelaySettings(RelaySettingsUpdate.Normal(update))
            }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages.filterIsInstance<Request.SetProviders>().collect {
                request ->
                val update =
                    RelayConstraintsUpdate(
                        location = null,
                        providers = request.providers,
                        ownership = null,
                        wireguardConstraints = null
                    )
                daemon.await().updateRelaySettings(RelaySettingsUpdate.Normal(update))
            }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages.filterIsInstance<Request.FetchRelayList>().collect {
                relayList = daemon.await().getRelayLocations()
            }
        }
    }

    fun onDestroy() {
        daemon.unregisterListener(this)
        scope.cancel()
    }

    private fun setUpListener(daemon: MullvadDaemon) {
        daemon.onRelayListChange = { relayLocations -> relayList = relayLocations }
    }

    private fun fetchInitialRelayList(daemon: MullvadDaemon) {
        synchronized(this) {
            if (relayList == null) {
                relayList = daemon.getRelayLocations()
            }
        }
    }
}
