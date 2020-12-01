package net.mullvad.mullvadvpn.service

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
import net.mullvad.talpid.ConnectivityListener
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class LocationInfoCache(
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener
) {
    private val fetchRequestChannel = runFetcher()

    private var lastKnownRealLocation: GeoIpLocation? = null
    private var selectedRelayLocation: GeoIpLocation? = null

    var onNewLocation: ((GeoIpLocation?) -> Unit)? = null
        set(value) {
            field = value
            value?.invoke(location)
        }

    var location: GeoIpLocation? = null
        set(value) {
            field = value
            onNewLocation?.invoke(value)
        }

    var state: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            when (value) {
                is TunnelState.Disconnected -> {
                    location = lastKnownRealLocation
                    fetchRequestChannel.sendBlocking(true)
                }
                is TunnelState.Connecting -> location = value.location
                is TunnelState.Connected -> {
                    location = value.location
                    fetchRequestChannel.sendBlocking(false)
                }
                is TunnelState.Disconnecting -> {
                    when (value.actionAfterDisconnect) {
                        ActionAfterDisconnect.Nothing -> location = lastKnownRealLocation
                        ActionAfterDisconnect.Block -> location = null
                        ActionAfterDisconnect.Reconnect -> location = selectedRelayLocation
                    }
                }
                is TunnelState.Error -> location = null
            }
        }

    var selectedRelay: RelayItem? = null
        set(value) {
            if (field != value) {
                field = value
                updateSelectedRelayLocation(value)
            }
        }

    init {
        connectivityListener.connectivityNotifier.subscribe(this) { isConnected ->
            if (isConnected && state is TunnelState.Disconnected) {
                fetchRequestChannel.sendBlocking(true)
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

    private fun runFetcher() = GlobalScope.actor<Boolean>(Dispatchers.Default, Channel.CONFLATED) {
        try {
            fetcherLoop(channel)
        } catch (exception: ClosedReceiveChannelException) {
        }
    }

    private suspend fun fetcherLoop(channel: ReceiveChannel<Boolean>) {
        val delays = ExponentialBackoff().apply {
            scale = 50
            cap = 30 /* min */ * 60 /* s */ * 1000 /* ms */
            count = 17 // ceil(log2(cap / scale) + 1)
        }

        while (true) {
            var isRealLocation = channel.receive()
            var newLocation = daemon.getCurrentLocation()

            while (newLocation == null || !channel.isEmpty) {
                isRealLocation = delayOrReceive(delays, channel, isRealLocation)
                newLocation = daemon.getCurrentLocation()
            }

            handleNewLocation(newLocation, isRealLocation)
            delays.reset()
        }
    }

    private suspend fun delayOrReceive(
        delays: ExponentialBackoff,
        channel: ReceiveChannel<Boolean>,
        currentValue: Boolean
    ): Boolean {
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

    private fun handleNewLocation(newLocation: GeoIpLocation, isRealLocation: Boolean) {
        if (isRealLocation) {
            lastKnownRealLocation = newLocation
        }

        location = newLocation
    }
}
