package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.dataproxy.LocationInfoCache

class LocationInfo(val parentView: View, val locationInfoCache: LocationInfoCache) {
    private val countryLabel: TextView = parentView.findViewById(R.id.country)
    private val cityLabel: TextView = parentView.findViewById(R.id.city)
    private val hostnameLabel: TextView = parentView.findViewById(R.id.hostname)

    private var updateJob: Job? = null

    init {
        locationInfoCache.onNewLocation = { country, city, hostname ->
            updateJob?.cancel()
            updateJob = updateViews(country, city, hostname)
        }
    }

    fun onDestroy() {
        updateJob?.cancel()
        locationInfoCache.onNewLocation = null
    }

    fun updateViews(country: String, city: String, hostname: String) =
            GlobalScope.launch(Dispatchers.Main) {
        countryLabel.text = country
        cityLabel.text = city
        hostnameLabel.text = hostname
    }
}
