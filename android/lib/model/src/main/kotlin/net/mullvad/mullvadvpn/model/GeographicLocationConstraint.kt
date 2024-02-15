package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class GeographicLocationConstraint : Parcelable {
    abstract val location: GeoIpLocation

    @Parcelize
    data class Country(val countryCode: String) : GeographicLocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, null, 0.0, 0.0, null)
    }

    @Parcelize
    data class City(val countryCode: String, val cityCode: String) :
        GeographicLocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, cityCode, 0.0, 0.0, null)
    }

    @Parcelize
    data class Hostname(val countryCode: String, val cityCode: String, val hostname: String) :
        GeographicLocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, cityCode, 0.0, 0.0, hostname)
    }
}
