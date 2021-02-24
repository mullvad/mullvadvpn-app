package net.mullvad.mullvadvpn.model

sealed class LocationConstraint(val code: Array<String>) {
    data class Country(val countryCode: String) : LocationConstraint(arrayOf(countryCode))

    data class City(val countryCode: String, val cityCode: String) :
        LocationConstraint(arrayOf(countryCode, cityCode))

    data class Hostname(val countryCode: String, val cityCode: String, val hostname: String) :
        LocationConstraint(arrayOf(countryCode, cityCode, hostname))
}
