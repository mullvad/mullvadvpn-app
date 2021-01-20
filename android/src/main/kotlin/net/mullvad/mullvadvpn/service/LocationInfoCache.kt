package net.mullvad.mullvadvpn.service

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
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.util.ExponentialBackoff
import net.mullvad.mullvadvpn.util.Intermittent
import net.mullvad.talpid.ConnectivityListener
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class LocationInfoCache(
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener,
    val daemon: Intermittent<MullvadDaemon>
) {
    companion object {
        private enum class RequestFetch {
            ForRealLocation,
            ForRelayLocation,
        }
    }

    private val fetchRequestChannel = runFetcher()

    private var lastKnownRealLocation: GeoIpLocation? = null
    private var selectedRelayLocation: GeoIpLocation? = null

    var onNewLocation by observable<((GeoIpLocation?) -> Unit)?>(null) { _, _, callback ->
        callback?.invoke(location)
    }

    var location: GeoIpLocation? by observable(null) { _, _, newLocation ->
        onNewLocation?.invoke(newLocation)
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

    var selectedRelay by observable<RelayItem?>(null) { _, oldRelay, newRelay ->
        if (newRelay != oldRelay) {
            updateSelectedRelayLocation(newRelay)
        }
    }

    init {
        connectivityListener.connectivityNotifier.subscribe(this) { isConnected ->
            if (isConnected && state is TunnelState.Disconnected) {
                fetchRequestChannel.sendBlocking(RequestFetch.ForRealLocation)
            }
        }

        connectionProxy.onStateChange.subscribe(this) { realState ->
            state = realState
        }
    }

    fun onDestroy() {
        connectivityListener.connectivityNotifier.unsubscribe(this)
        connectionProxy.onStateChange.unsubscribe(this)
        fetchRequestChannel.close()

        onNewLocation = null
    }

    private fun updateSelectedRelayLocation(relayItem: RelayItem?) {
        selectedRelayLocation = when (relayItem) {
            is RelayCountry -> GeoIpLocation(null, null, relayItem.name, null, null)
            is RelayCity -> GeoIpLocation(
                null,
                null,
                relayItem.country.name,
                relayItem.name,
                null
            )
            is Relay -> GeoIpLocation(
                null,
                null,
                relayItem.city.country.name,
                relayItem.city.name,
                relayItem.name
            )
            else -> null
        }
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
        val delays = ExponentialBackoff().apply {
            scale = 50
            cap = 30 /* min */ * 60 /* s */ * 1000 /* ms */
            count = 17 // ceil(log2(cap / scale) + 1)
        }

        while (true) {
            var fetchType = channel.receive()
            var newLocation = daemon.await().getCurrentLocation()

            while (newLocation == null || !channel.isEmpty) {
                fetchType = delayOrReceive(delays, channel, fetchType)
                newLocation = daemon.await().getCurrentLocation()
            }

            handleNewLocation(newLocation, fetchType)
            delays.reset()
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
}
