package net.mullvad.mullvadvpn

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache

class LocationInfo(val parentView: View, val locationInfoCache: LocationInfoCache) {
    private val countryLabel: TextView = parentView.findViewById(R.id.country)
    private val cityLabel: TextView = parentView.findViewById(R.id.city)
    private val hostnameLabel: TextView = parentView.findViewById(R.id.hostname)

    init {
        locationInfoCache.onNewLocation = { country, city, hostname ->
            updateViews(country, city, hostname)
        }
    }

    fun onDestroy() {
        locationInfoCache.onNewLocation = null
    }

    fun updateViews(country: String, city: String, hostname: String) {
        countryLabel.text = country
        cityLabel.text = city
        hostnameLabel.text = hostname
    }
}
