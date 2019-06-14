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

    var location: GeoIpLocation? = null
        set(value) {
            field = value
            notifyNewLocation(value)
        }

    fun setState(state: TunnelStateTransition) {
        activeFetch?.cancel()
        activeFetch = null

        when (state) {
            is TunnelStateTransition.Disconnected -> activeFetch = fetchRealLocation()
            is TunnelStateTransition.Connecting -> activeFetch = fetchRelayLocation()
            is TunnelStateTransition.Connected -> activeFetch = fetchRelayLocation()
            is TunnelStateTransition.Disconnecting -> location = lastKnownRealLocation
            is TunnelStateTransition.Blocked -> location = null
        }
    }

    fun notifyNewLocation(location: GeoIpLocation?) {
        val country = location?.country ?: ""
        val city = location?.city ?: ""
        val hostname = location?.hostname ?: ""

        onNewLocation?.invoke(country, city, hostname)
    }

    private fun fetchRealLocation() = GlobalScope.launch(Dispatchers.Main) {
        var realLocation: GeoIpLocation? = null
        var remainingAttempts = 10

        while (realLocation == null && remainingAttempts > 0) {
            realLocation = fetchLocation().await()
            remainingAttempts -= 1
        }

        lastKnownRealLocation = realLocation
        location = realLocation
    }

    private fun fetchRelayLocation() = GlobalScope.launch(Dispatchers.Main) {
        location = fetchLocation().await()
    }

    private fun fetchLocation() = GlobalScope.async(Dispatchers.Default) {
        daemon.await().getCurrentLocation()
    }
}
