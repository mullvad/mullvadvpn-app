package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.MullvadDaemon
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.Relay
import net.mullvad.mullvadvpn.relaylist.RelayCity
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.talpid.ConnectivityListener
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

class LocationInfoCache(
    val daemon: Deferred<MullvadDaemon>,
    val connectivityListener: Deferred<ConnectivityListener>,
    val relayListListener: RelayListListener
) {
    private var lastKnownRealLocation: GeoIpLocation? = null
    private var activeFetch: Job? = null

    private val connectivityListenerId = GlobalScope.async(Dispatchers.Default) {
        connectivityListener.await().connectivityNotifier.subscribe { isConnected ->
            if (isConnected) {
                fetchLocation()
            }
        }
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
                        ActionAfterDisconnect.Reconnect -> location = locationFromSelectedRelay()
                    }
                }
                is TunnelState.Error -> location = null
            }
        }

    private fun locationFromSelectedRelay(): GeoIpLocation? {
        val relayItem = relayListListener.selectedRelayItem

        when (relayItem) {
            is RelayCountry -> return GeoIpLocation(null, null, relayItem.name, null, null)
            is RelayCity -> return GeoIpLocation(
                null,
                null,
                relayItem.country.name,
                relayItem.name,
                null
            )
            is Relay -> return GeoIpLocation(
                null,
                null,
                relayItem.city.country.name,
                relayItem.city.name,
                relayItem.name
            )
        }

        return null
    }

    private fun fetchLocation() {
        val previousFetch = activeFetch
        val initialState = state

        activeFetch = GlobalScope.launch(Dispatchers.Main) {
            var newLocation: GeoIpLocation? = null

            previousFetch?.join()

            while (newLocation == null && shouldRetryFetch() && state == initialState) {
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
        daemon.await().getCurrentLocation()
    }

    private suspend fun shouldRetryFetch(): Boolean {
        val state = this.state

        return connectivityListener.await().isConnected &&
            (state is TunnelState.Disconnected || state is TunnelState.Connected)
    }
}
