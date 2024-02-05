package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
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
                        getCurrentRelayConstraints()
                            .copy(location = Constraint.Only(request.locationConstraint))
                    daemon.await().setRelaySettings(RelaySettings.Normal(update))
                }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.SetWireguardConstraints>()
                .collect { request ->
                    val update =
                        getCurrentRelayConstraints()
                            .copy(wireguardConstraints = request.wireguardConstraints)
                    daemon.await().setRelaySettings(RelaySettings.Normal(update))
                }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages.filterIsInstance<Request.FetchRelayList>().collect {
                relayList = daemon.await().getRelayLocations()
            }
        }

        scope.launch {
            endpoint.dispatcher.parsedMessages
                .filterIsInstance<Request.SetOwnershipAndProviders>()
                .collect { request ->
                    val update =
                        getCurrentRelayConstraints()
                            .copy(ownership = request.ownership, providers = request.providers)
                    daemon.await().setRelaySettings(RelaySettings.Normal(update))
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

    private suspend fun getCurrentRelayConstraints(): RelayConstraints =
        when (val relaySettings = daemon.await().getSettings()?.relaySettings) {
            is RelaySettings.Normal -> relaySettings.relayConstraints
            else ->
                RelayConstraints(
                    location = Constraint.Any(),
                    providers = Constraint.Any(),
                    ownership = Constraint.Any(),
                    wireguardConstraints = WireguardConstraints(Constraint.Any())
                )
        }
}
