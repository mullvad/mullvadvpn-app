package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.MullvadDaemon

class LocationInfoCache(val daemon: Deferred<MullvadDaemon>) {
    private var lastKnownRealLocation: GeoIpLocation? = null
    private var activeFetch: Job? = null

    var onNewLocation: ((String, String, String) -> Unit)? = null
        set(value) {
            field = value
            notifyNewLocation()
        }

    var location: GeoIpLocation? = null
        set(value) {
            field = value
            notifyNewLocation()
        }

    var state: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            when (value) {
                is TunnelState.Disconnected -> fetchLocation()
                is TunnelState.Connecting -> location = value.location
                is TunnelState.Connected -> {
                    location = value.location
                    fetchLocation()
                }
                is TunnelState.Disconnecting -> location = lastKnownRealLocation
                is TunnelState.Blocked -> location = null
            }
        }

    fun notifyNewLocation() {
        val location = this.location
        val country = location?.country ?: ""
        val city = location?.city ?: ""
        val hostname = location?.hostname ?: ""

        onNewLocation?.invoke(country, city, hostname)
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

    private fun shouldRetryFetch(): Boolean {
        val state = this.state

        return state is TunnelState.Disconnected ||
            state is TunnelState.Connected
    }
}
