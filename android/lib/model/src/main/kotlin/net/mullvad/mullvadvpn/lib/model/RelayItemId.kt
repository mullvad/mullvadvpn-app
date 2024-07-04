package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@optics
sealed interface RelayItemId : Parcelable {
    companion object
}

@optics
@Parcelize
@JvmInline
value class CustomListId(val value: String) : RelayItemId, Parcelable {
    companion object
}

@optics
@Parcelize
sealed interface GeoLocationId : RelayItemId, Parcelable {
    @optics
    @Parcelize
    data class Country(val countryCode: String) : GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class City(val countryCode: Country, val cityCode: String) : GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class Hostname(val city: City, val hostname: String) : GeoLocationId {
        companion object
    }

    val country: Country
        get() =
            when (this) {
                is Country -> this
                is City -> this.countryCode
                is Hostname -> this.city.countryCode
            }

    companion object
}
