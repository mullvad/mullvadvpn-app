package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.IgnoredOnParcel
import kotlinx.parcelize.Parcelize

typealias DomainCustomList = CustomList

sealed interface HopSelection {
    data class Single(val relay: Constraint<RelayItem>?) : HopSelection

    data class Multi(val entry: Constraint<RelayItem>?, val exit: Constraint<RelayItem>?) :
        HopSelection

    fun entry() =
        when (this) {
            is Multi -> entry
            is Single -> relay
        }

    fun exit() =
        when (this) {
            is Multi -> exit
            is Single -> relay
        }
}

@Parcelize
@optics
sealed interface RelayItem : Parcelable {
    val id: RelayItemId
    val name: String

    val active: Boolean
    val hasChildren: Boolean

    @optics
    data class CustomList(val customList: DomainCustomList, val locations: List<Location>) :
        RelayItem {
        @IgnoredOnParcel override val name: String = customList.name.value
        @IgnoredOnParcel override val id = customList.id

        @IgnoredOnParcel override val active = locations.any { it.active }
        @IgnoredOnParcel override val hasChildren: Boolean = locations.isNotEmpty()

        companion object
    }

    @Parcelize
    @optics
    sealed interface Location : RelayItem {
        override val id: GeoLocationId

        @optics
        data class Country(
            override val id: GeoLocationId.Country,
            override val name: String,
            val cities: List<City>,
        ) : Location {
            @IgnoredOnParcel val relays = cities.flatMap { city -> city.relays }
            @IgnoredOnParcel override val active = cities.any { it.active }
            @IgnoredOnParcel override val hasChildren: Boolean = cities.isNotEmpty()

            companion object
        }

        @optics
        data class City(
            override val id: GeoLocationId.City,
            override val name: String,
            val relays: List<Relay>,
            val countryName: String,
        ) : Location {
            @IgnoredOnParcel override val active = relays.any { it.active }
            @IgnoredOnParcel override val hasChildren: Boolean = relays.isNotEmpty()

            companion object
        }

        @optics
        data class Relay(
            override val id: GeoLocationId.Hostname,
            override val active: Boolean,
            val provider: ProviderId,
            val ownership: Ownership,
            val daita: Boolean,
            val quic: Quic?,
            val lwo: Boolean,
            val cityName: String,
            val countryName: String,
        ) : Location {
            @IgnoredOnParcel override val name: String = id.code
            @IgnoredOnParcel override val hasChildren: Boolean = false

            companion object
        }

        companion object
    }

    companion object
}
