package net.mullvad.mullvadvpn.dataproxy

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelStateTransition
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

    var state: TunnelStateTransition = TunnelStateTransition.Disconnected()
        set(value) {
            field = value

            when (value) {
                is TunnelStateTransition.Disconnected -> fetchLocation()
                is TunnelStateTransition.Connecting -> fetchLocation()
                is TunnelStateTransition.Connected -> fetchLocation()
                is TunnelStateTransition.Disconnecting -> location = lastKnownRealLocation
                is TunnelStateTransition.Blocked -> location = null
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
                    is TunnelStateTransition.Disconnected -> {
                        lastKnownRealLocation = newLocation
                        location = newLocation
                    }
                    is TunnelStateTransition.Connecting -> location = newLocation
                    is TunnelStateTransition.Connected -> location = newLocation
                }
            }
        }
    }

    private fun executeFetch() = GlobalScope.async(Dispatchers.Default) {
        daemon.await().getCurrentLocation()
    }

    private fun shouldRetryFetch(): Boolean {
        val state = this.state

        return state is TunnelStateTransition.Disconnected ||
            state is TunnelStateTransition.Connected
    }
}
