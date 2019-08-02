package net.mullvad.mullvadvpn

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.GeoIpLocation

class LocationInfo(val parentView: View) {
    private val countryLabel: TextView = parentView.findViewById(R.id.country)
    private val cityLabel: TextView = parentView.findViewById(R.id.city)
    private val hostnameLabel: TextView = parentView.findViewById(R.id.hostname)

    var location: GeoIpLocation? = null
        set(value) {
            countryLabel.text = value?.country ?: ""
            cityLabel.text = value?.city ?: ""
            hostnameLabel.text = value?.hostname ?: ""
        }
}
