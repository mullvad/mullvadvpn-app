package net.mullvad.mullvadvpn

import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelStateTransition

class LocationInfo(val parentView: View, val daemon: Deferred<MullvadDaemon>) {
    private val country: TextView = parentView.findViewById(R.id.country)
    private val city: TextView = parentView.findViewById(R.id.city)
    private val hostname: TextView = parentView.findViewById(R.id.hostname)

    private var lastKnownRealLocation: GeoIpLocation? = null

    var location: GeoIpLocation? = null
        set(value) {
            field = value
            updateViews(value)
        }

    fun setState(state: TunnelStateTransition) {
        when (state) {
            is TunnelStateTransition.Disconnected -> fetchRealLocation()
            is TunnelStateTransition.Connecting -> fetchRelayLocation()
            is TunnelStateTransition.Connected -> fetchRelayLocation()
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
        val realLocation = fetchLocation().await()

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
