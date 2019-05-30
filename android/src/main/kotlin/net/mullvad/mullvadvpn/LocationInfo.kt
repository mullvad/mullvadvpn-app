package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelStateTransition

class LocationInfo(val parentView: View, val daemon: Deferred<MullvadDaemon>) {
    private val country: TextView = parentView.findViewById(R.id.country)
    private val city: TextView = parentView.findViewById(R.id.city)
    private val hostname: TextView = parentView.findViewById(R.id.hostname)

    private var lastKnownRealLocation: GeoIpLocation? = null

    private var activeFetch: Job? = null

    var location: GeoIpLocation? = null
        set(value) {
            field = value
            updateViews(value)
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

    fun updateViews(location: GeoIpLocation?) {
        country.text = location?.country ?: ""
        city.text = location?.city ?: ""
        hostname.text = location?.hostname ?: ""
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
