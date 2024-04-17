package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class GeographicLocationConstraint : Parcelable {

    @Parcelize data class Country(val countryCode: String) : GeographicLocationConstraint()

    @Parcelize
    data class City(val countryCode: String, val cityCode: String) : GeographicLocationConstraint()

    @Parcelize
    data class Hostname(val countryCode: String, val cityCode: String, val hostname: String) :
        GeographicLocationConstraint()
}
