package net.mullvad.mullvadvpn

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache
import net.mullvad.mullvadvpn.model.GeoIpLocation

class LocationInfo(val parentView: View, val locationInfoCache: LocationInfoCache) {
    private val countryLabel: TextView = parentView.findViewById(R.id.country)
    private val cityLabel: TextView = parentView.findViewById(R.id.city)
    private val hostnameLabel: TextView = parentView.findViewById(R.id.hostname)

    init {
        locationInfoCache.onNewLocation = { newLocation ->
            location = newLocation
        }
    }

    var location: GeoIpLocation? = null
        set(value) {
            countryLabel.text = value?.country ?: ""
            cityLabel.text = value?.city ?: ""
            hostnameLabel.text = value?.hostname ?: ""
        }

    fun onDestroy() {
        locationInfoCache.onNewLocation = null
    }
}
