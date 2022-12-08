package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.ExponentialBackoff
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class LocationInfoCache(private val endpoint: ServiceEndpoint) {
    companion object {
        private enum class RequestFetch {
            ForRealLocation,
            ForRelayLocation,
        }
    }

    private val fetchRetryDelays = ExponentialBackoff().apply {
        scale = 50
        cap = 30 /* min */ * 60 /* s */ * 1000 /* ms */
        count = 17 // ceil(log2(cap / scale) + 1)
    }

    private val fetchRequestChannel = runFetcher()

    private val daemon
        get() = endpoint.intermittentDaemon

    private var lastKnownRealLocation: GeoIpLocation? = null
    private var selectedRelayLocation: GeoIpLocation? = null

    var location: GeoIpLocation? by observable(null) { _, _, newLocation ->
        endpoint.sendEvent(Event.NewLocation(newLocation))
    }

    var state by observable<TunnelState>(TunnelState.Disconnected) { _, _, newState ->
        when (newState) {
            is TunnelState.Disconnected -> {
                location = lastKnownRealLocation
                fetchRequestChannel.trySendBlocking(RequestFetch.ForRealLocation)
            }
            is TunnelState.Connecting -> location = newState.location
            is TunnelState.Connected -> {
                location = newState.location
                fetchRequestChannel.trySendBlocking(RequestFetch.ForRelayLocation)
            }
            is TunnelState.Disconnecting -> {
                when (newState.actionAfterDisconnect) {
                    ActionAfterDisconnect.Nothing -> location = lastKnownRealLocation
                    ActionAfterDisconnect.Block -> location = null
                    ActionAfterDisconnect.Reconnect -> location = selectedRelayLocation
                }
            }
            is TunnelState.Error -> location = null
        }
    }

    init {
        endpoint.connectionProxy.onStateChange.subscribe(this) { newState ->
            state = newState
        }

        endpoint.connectivityListener.connectivityNotifier.subscribe(this) { isConnected ->
            if (isConnected && state is TunnelState.Disconnected) {
                fetchRequestChannel.trySendBlocking(RequestFetch.ForRealLocation)
            }
        }

        endpoint.settingsListener.relaySettingsNotifier.subscribe(this, ::updateSelectedLocation)
    }

    fun onDestroy() {
        endpoint.connectionProxy.onStateChange.unsubscribe(this)
        endpoint.connectivityListener.connectivityNotifier.unsubscribe(this)
        endpoint.settingsListener.relaySettingsNotifier.unsubscribe(this)

        fetchRequestChannel.close()
    }

    private fun runFetcher() = GlobalScope.actor<RequestFetch>(
        Dispatchers.Default,
        Channel.CONFLATED
    ) {
        try {
            fetcherLoop(channel)
        } catch (exception: ClosedReceiveChannelException) {
        }
    }

    private suspend fun fetcherLoop(channel: ReceiveChannel<RequestFetch>) {
        channel.receiveAsFlow()
            .flatMapLatest(::fetchCurrentLocation)
            .collect(::handleFetchedLocation)
    }

    private fun fetchCurrentLocation(fetchType: RequestFetch) = flow {
        var newLocation = daemon.await().getCurrentLocation()

        fetchRetryDelays.reset()

        while (newLocation == null) {
            delay(fetchRetryDelays.next())
            newLocation = daemon.await().getCurrentLocation()
        }

        emit(Pair(newLocation, fetchType))
    }

    private suspend fun handleFetchedLocation(pairItem: Pair<GeoIpLocation, RequestFetch>) {
        val (newLocation, fetchType) = pairItem

        if (fetchType == RequestFetch.ForRealLocation) {
            lastKnownRealLocation = newLocation
        }

        location = newLocation
    }

    private fun updateSelectedLocation(relaySettings: RelaySettings?) {
        val settings = relaySettings as? RelaySettings.Normal
        val constraint = settings?.relayConstraints?.location as? Constraint.Only

        selectedRelayLocation = constraint?.value?.location
    }
}
