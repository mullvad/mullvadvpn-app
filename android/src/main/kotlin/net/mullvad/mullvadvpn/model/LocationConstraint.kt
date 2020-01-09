package net.mullvad.mullvadvpn.model

sealed class LocationConstraint(val code: Array<String>) {
    class Country(var countryCode: String) : LocationConstraint(arrayOf(countryCode)) {
        fun get0() = countryCode
    }

    class City(var countryCode: String, var cityCode: String) :
        LocationConstraint(arrayOf(countryCode, cityCode))
    {
        fun get0() = countryCode
        fun get1() = cityCode
    }

    class Hostname(var countryCode: String, var cityCode: String, var hostname: String) :
        LocationConstraint(arrayOf(countryCode, cityCode, hostname))
    {
        fun get0() = countryCode
        fun get1() = cityCode
        fun get2() = hostname
    }
}
