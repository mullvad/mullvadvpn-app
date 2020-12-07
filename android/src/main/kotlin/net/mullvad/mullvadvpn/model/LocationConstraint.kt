package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class LocationConstraint(val code: Array<String>) : Parcelable {
    @Parcelize
    class Country(var countryCode: String) : LocationConstraint(arrayOf(countryCode)), Parcelable {
        fun get0() = countryCode
    }

    @Parcelize
    class City(var countryCode: String, var cityCode: String) :
        LocationConstraint(arrayOf(countryCode, cityCode)), Parcelable {
        fun get0() = countryCode
        fun get1() = cityCode
    }

    @Parcelize
    class Hostname(var countryCode: String, var cityCode: String, var hostname: String) :
        LocationConstraint(arrayOf(countryCode, cityCode, hostname)), Parcelable {
        fun get0() = countryCode
        fun get1() = cityCode
        fun get2() = hostname
    }
}
