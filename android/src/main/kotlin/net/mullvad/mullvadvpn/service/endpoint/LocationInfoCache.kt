package net.mullvad.mullvadvpn.service.endpoint

import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.ExponentialBackoff
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.util.autoSubscribable

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
                fetchRequestChannel.sendBlocking(RequestFetch.ForRealLocation)
            }
            is TunnelState.Connecting -> location = newState.location
            is TunnelState.Connected -> {
                location = newState.location
                fetchRequestChannel.sendBlocking(RequestFetch.ForRelayLocation)
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

    var stateEvents by autoSubscribable<TunnelState>(this, TunnelState.Disconnected) { newState ->
        state = newState
    }

    init {
        endpoint.connectivityListener.connectivityNotifier.subscribe(this) { isConnected ->
            if (isConnected && state is TunnelState.Disconnected) {
                fetchRequestChannel.sendBlocking(RequestFetch.ForRealLocation)
            }
        }

        endpoint.settingsListener.relaySettingsNotifier.subscribe(this, ::updateSelectedLocation)
    }

    fun onDestroy() {
        endpoint.connectivityListener.connectivityNotifier.unsubscribe(this)
        endpoint.settingsListener.relaySettingsNotifier.unsubscribe(this)
        stateEvents = null

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
        while (true) {
            var fetchType = channel.receive()
            var newLocation = daemon.await().getCurrentLocation()

            while (newLocation == null || !channel.isEmpty) {
                fetchType = delayOrReceive(fetchRetryDelays, channel, fetchType)
                newLocation = daemon.await().getCurrentLocation()
            }

            handleNewLocation(newLocation, fetchType)
            fetchRetryDelays.reset()
        }
    }

    private suspend fun delayOrReceive(
        delays: ExponentialBackoff,
        channel: ReceiveChannel<RequestFetch>,
        currentValue: RequestFetch
    ): RequestFetch {
        try {
            val newValue = withTimeout(delays.next()) {
                channel.receive()
            }

            delays.reset()

            return newValue
        } catch (timeOut: TimeoutCancellationException) {
            return currentValue
        }
    }

    private fun handleNewLocation(newLocation: GeoIpLocation, fetchType: RequestFetch) {
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
