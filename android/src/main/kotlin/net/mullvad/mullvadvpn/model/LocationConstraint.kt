package net.mullvad.mullvadvpn.model

sealed class LocationConstraint(val code: Array<String>) {
    class Country(var countryCode: String) : LocationConstraint(arrayOf(countryCode))

    class City(var countryCode: String, var cityCode: String)
        : LocationConstraint(arrayOf(countryCode, cityCode))

    class Hostname(var countryCode: String, var cityCode: String, var hostname: String)
        : LocationConstraint(arrayOf(countryCode, cityCode, hostname))
}
