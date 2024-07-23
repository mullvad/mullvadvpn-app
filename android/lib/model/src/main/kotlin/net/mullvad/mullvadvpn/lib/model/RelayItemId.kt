package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.IgnoredOnParcel
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
    data class Country(override val code: String) : GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class City(override val country: Country, override val code: String) : GeoLocationId {
        companion object
    }

    @optics
    @Parcelize
    data class Hostname(val city: City, override val code: String) : GeoLocationId {
        companion object
    }

    val code: String
    val country: Country
        get() =
            when (this) {
                is Country -> this
                is City -> this.country
                is Hostname -> this.city.country
            }

    companion object
}
