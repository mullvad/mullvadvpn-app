package net.mullvad.mullvadvpn.service

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.talpid.ConnectivityListener
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

const val DELAY_SCALE: Long = 50
const val MAX_DELAY: Long = 30 * 60 * 1000
const val MAX_RETRIES: Int = 17 // ceil(log2(MAX_DELAY / DELAY_SCALE) + 1)

class LocationInfoCache(
    val daemon: MullvadDaemon,
    val connectionProxy: ConnectionProxy,
    val connectivityListener: ConnectivityListener
) {
    private var activeFetch: Job? = null
    private var lastKnownRealLocation: GeoIpLocation? = null
    private var selectedRelayLocation: GeoIpLocation? = null

    private val connectivityListenerId =
        connectivityListener.connectivityNotifier.subscribe { isConnected ->
            if (isConnected && state is TunnelState.Disconnected) {
                fetchLocation()
            }
        }

    private val realStateListenerId = connectionProxy.onStateChange.subscribe { realState ->
        state = realState
    }

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
                    fetchLocation()
                }
                is TunnelState.Connecting -> location = value.location
                is TunnelState.Connected -> {
                    location = value.location
                    fetchLocation()
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

    fun onDestroy() {
        connectivityListener.connectivityNotifier.unsubscribe(connectivityListenerId)
        connectionProxy.onStateChange.unsubscribe(realStateListenerId)
        activeFetch?.cancel()
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

    private fun fetchLocation() {
        val previousFetch = activeFetch
        val initialState = state

        activeFetch = GlobalScope.launch(Dispatchers.Main) {
            var newLocation: GeoIpLocation? = null
            var retry = 0

            previousFetch?.join()

            while (newLocation == null && shouldRetryFetch() && state == initialState) {
                delayFetch(retry)
                retry += 1

                newLocation = executeFetch().await()
            }

            if (newLocation != null && state == initialState) {
                when (state) {
                    is TunnelState.Disconnected -> {
                        lastKnownRealLocation = newLocation
                        location = newLocation
                    }
                    is TunnelState.Connecting -> location = newLocation
                    is TunnelState.Connected -> location = newLocation
                }
            }
        }
    }

    private fun executeFetch() = GlobalScope.async(Dispatchers.Default) {
        daemon.getCurrentLocation()
    }

    private suspend fun delayFetch(retryAttempt: Int) {
        var duration = 0L

        // The first attempt has no delay
        if (retryAttempt >= MAX_RETRIES) {
            duration = MAX_DELAY
        } else if (retryAttempt >= 1) {
            val exponent = retryAttempt - 1
            duration = (1L shl exponent) * DELAY_SCALE
        }

        delay(duration)
    }

    private fun shouldRetryFetch(): Boolean {
        val state = this.state

        return connectivityListener.isConnected &&
            (state is TunnelState.Disconnected || state is TunnelState.Connected)
    }
}
