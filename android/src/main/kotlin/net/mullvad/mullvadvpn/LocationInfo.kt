package net.mullvad.mullvadvpn

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.GeoIpLocation

class LocationInfo(val parentView: View) {
    private val country: TextView = parentView.findViewById(R.id.country)
    private val city: TextView = parentView.findViewById(R.id.city)
    private val hostname: TextView = parentView.findViewById(R.id.hostname)

    var location: GeoIpLocation? = null
        set(value) {
            country.text = value?.country ?: ""
            city.text = value?.city ?: ""
            hostname.text = value?.hostname ?: ""
        }
}
