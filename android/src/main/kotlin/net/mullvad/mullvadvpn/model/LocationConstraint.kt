package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class LocationConstraint(val code: Array<String>) : Parcelable {
    @Parcelize
    data class Country(val countryCode: String) :
        LocationConstraint(arrayOf(countryCode)), Parcelable

    @Parcelize
    data class City(val countryCode: String, val cityCode: String) :
        LocationConstraint(arrayOf(countryCode, cityCode)), Parcelable

    @Parcelize
    data class Hostname(val countryCode: String, val cityCode: String, val hostname: String) :
        LocationConstraint(arrayOf(countryCode, cityCode, hostname)), Parcelable
}
