package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class LocationConstraint : Parcelable {
    @Parcelize
    data class Country(val countryCode: String) : LocationConstraint(), Parcelable

    @Parcelize
    data class City(val countryCode: String, val cityCode: String) :
        LocationConstraint(), Parcelable

    @Parcelize
    data class Hostname(val countryCode: String, val cityCode: String, val hostname: String) :
        LocationConstraint(), Parcelable
}
