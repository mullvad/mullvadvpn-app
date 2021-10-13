package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class LocationConstraint : Parcelable {
    abstract val location: GeoIpLocation

    @Parcelize
    data class Country(val countryCode: String) : LocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, null, null)
    }

    @Parcelize
    data class City(val countryCode: String, val cityCode: String) : LocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, cityCode, null)
    }

    @Parcelize
    data class Hostname(
        val countryCode: String,
        val cityCode: String,
        val hostname: String
    ) : LocationConstraint() {
        override val location: GeoIpLocation
            get() = GeoIpLocation(null, null, countryCode, cityCode, hostname)
    }
}
